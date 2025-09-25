use std::io::Write;

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

	// unpack zip /assets/minecraft/textures
	// loop through entries
	//    if (entry is mcmeta) save_to ./out/{version_id}/entry.path
	//    img = load_png
	//    if (/\/(?:block|optifine|painting)\//.exec(path)) wrap = true
	//    if (/\/(?:model|entity)\//.exec(path)) relayer = true
	//    out_img = process::process(img, wrap, relayer)
	//    out_img = oxipng(out_img)
	//    save_to ./out/{version_id}/entry.path

	zip.finish()?;
	std::fs::remove_dir_all("./tmp").expect("failed to clean up minecraft assets");
	Ok(())
}
