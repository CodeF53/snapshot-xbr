mod mojang_api;
mod process;
mod find_pack_format;

#[tokio::main]
async fn main() {
    let version_id = "1.21.6";
    mojang_api::get_client_files(version_id).await.unwrap();

    println!("{version_id} has pack format {}", find_pack_format::find_pack_format().unwrap())
}