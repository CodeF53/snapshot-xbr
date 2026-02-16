use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize};
use std::io::{Read, Write};

#[derive(Deserialize)]
struct PackVersion {
	pack_version: ResourceVersion,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ResourceVersion {
	MajorResource { resource_major: u8 },
	Resource { resource: u8 },
}
impl ResourceVersion {
	fn to_string(self) -> String {
		match self {
			Self::MajorResource { resource_major } => resource_major.to_string(),
			Self::Resource { resource } => resource.to_string(),
		}
	}
}

pub fn package_zip(
	output_path: &str,
	mut client_jar: zip::ZipArchive<std::io::Cursor<bytes::Bytes>>,
) {
	if !std::fs::exists("./output/").expect("checking for output directory to work") {
		std::fs::create_dir("./output").expect("creating output directory to work");
	}
	let zip_file = std::fs::File::create(&output_path).expect("failed to create output zip");
	let mut zip = zip::ZipWriter::new(zip_file);
	let zip_options = zip::write::SimpleFileOptions::default()
		.compression_method(zip::CompressionMethod::Deflated);

	// write pack.mcmeta with correct pack format and pack.png to output zip
	let jar_version_inf = client_jar
		.by_name("version.json")
		.expect("client jar should contain version.json");

	let version_inf: PackVersion = serde_json::from_reader(jar_version_inf).unwrap();
	let pack_meta = &include_str!("./static/pack.mcmeta")
		.replace("PACK_FORMAT", &version_inf.pack_version.to_string());
	zip.start_file("pack.mcmeta", zip_options)
		.expect("adding pack.mcmeta to zip to work");
	zip.write_all(pack_meta.to_string().as_bytes())
		.expect("writing pack.mcmeta to zip to work");

	let pack_png = include_bytes!("./static/pack.png");
	zip.start_file("pack.png", zip_options)
		.expect("adding pack.png to zip to work");
	zip.write_all(pack_png)
		.expect("writing pack.png to zip to work");

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
		zip.add_directory_from_path(path, zip_options)
			.expect(&format!("adding directory (#{path}) to zip to work"));
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

				let proccessed_data = crate::process::process(file.1, tile, relayer);
				(file.0, proccessed_data)
			}
		})
		.collect::<Vec<_>>()
		.into_iter()
		.for_each(|file| {
			zip.start_file(file.0, zip_options)
				.expect(&format!("adding file ({}) to zip to work", file.0));
			zip.write_all(&file.1)
				.expect(&format!("writing file ({}) to zip to work", file.0));
		});

	zip.finish().expect("zip to sucessfully get saved");
}
