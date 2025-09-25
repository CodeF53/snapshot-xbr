mod find_pack_format;
mod mojang_api;
mod process;

#[tokio::main]
async fn main() {
	let version_id = "1.21.6";
	mojang_api::get_client_files(version_id).await.unwrap();

	// unpack zip /assets/minecraft/textures
	// loop through entries
	//    if (entry is mcmeta) save_to ./out/{version_id}/entry.path
	//    img = load_png
	//    if (/\/(?:block|optifine|painting)\//.exec(path)) wrap = true
	//    if (/\/(?:model|entity)\//.exec(path)) relayer = true
	//    out_img = process::process(img, wrap, relayer)
	//    out_img = oxipng(out_img)
	//    save_to ./out/{version_id}/entry.path

	// write pack.mcmeta with correct pack format and pack.png to output zip
	let pack_png = include_bytes!("./static/pack.png");
	let mut pack_meta = include_str!("./static/pack.mcmeta");
	let pack_format = find_pack_format::find_pack_format().expect("failed to find pack format");
	pack_meta = &pack_meta.replace("PACK_FORMAT", &pack_format.to_string());

	// zip everything

	// maybe auto upload to modrinth
}
