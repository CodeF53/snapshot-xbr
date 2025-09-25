use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::io::{Read, Write};

mod find_pack_format;
mod mojang_api;
mod process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let version_id = "1.21.6";
	mojang_api::get_client_files(version_id).await.unwrap();

	if !std::fs::exists("./output/")? {
		std::fs::create_dir("./output")?;
	}
	let zip_file = std::fs::File::create(format!("./output/{}.zip", version_id))
		.expect("failed to create output zip");
	let mut zip = zip::ZipWriter::new(zip_file);
	let zip_options = zip::write::SimpleFileOptions::default()
		.compression_method(zip::CompressionMethod::Deflated);

	// write pack.mcmeta with correct pack format and pack.png to output zip
	let pack_format = find_pack_format::find_pack_format().expect("failed to find pack format");
	let pack_meta =
		&include_str!("./static/pack.mcmeta").replace("PACK_FORMAT", &pack_format.to_string());
	zip.start_file("pack.mcmeta", zip_options)?;
	zip.write_all(pack_meta.to_string().as_bytes())?;

	let pack_png = include_bytes!("./static/pack.png");
	zip.start_file("pack.png", zip_options)?;
	zip.write_all(pack_png)?;

	//
	let mut client_jar = zip::ZipArchive::new(
		std::fs::File::open("./tmp/client.jar").expect("client.jar should exist at this point"),
	)
	.expect("jar file is valid zip archive");

	let wanted_paths = client_jar
		.file_names()
		.filter(|p| p.starts_with("assets/minecraft/textures/"))
		.filter(|&p| {
			!["font", "colormap", "gui/title", "gui/realms", "misc"]
				.iter()
				.any(|s| p.contains(s))
		})
		.filter(|&p| {
			!["clouds", "end_flash", "end_sky", "dither", "isles"]
				.iter()
				.any(|s| p.starts_with(&format!("{s}.png")))
		})
		.map(|s| s.to_string())
		.collect::<Vec<String>>();

	for path in &wanted_paths {
		if !path.ends_with('/') {
			continue;
		}
		zip.add_directory_from_path(path, zip_options)?;
	}

	wanted_paths
		.iter()
		.filter(|path| !path.ends_with('/'))
		.map(|path| {
			let mut file_zip = client_jar.by_name(path).unwrap();
			let mut file_bytes = Vec::new();
			file_zip.read_to_end(&mut file_bytes).unwrap();
			(path, file_bytes)
		})
		.collect::<Vec<_>>()
		.into_par_iter()
		.map(|file| {
			if file.0.ends_with("mcmeta") {
				file
			} else {
				let tile = ["/block/", "/optifine/", "/painting/"]
					.iter()
					.any(|f| file.0.contains(f));
				let relayer = ["/model/", "/entity/"].iter().any(|f| file.0.contains(f));

				let proccessed_data = process::process(file.1, tile, relayer);
				(file.0, proccessed_data)
			}
		})
		.collect::<Vec<_>>()
		.into_iter()
		.for_each(|file| {
			println!("{}", file.0);
			zip.start_file(file.0, zip_options).unwrap();
			zip.write_all(&file.1).unwrap();
		});

	zip.finish()?;
	std::fs::remove_dir_all("./tmp").expect("failed to clean up minecraft assets");
	Ok(())
}
