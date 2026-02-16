use clap::Parser;

mod discord_webhook;
mod modrinth_api;
mod mojang_api;
mod package_zip;
mod process;

#[derive(clap::Parser)]
#[command(arg_required_else_help(true))]
struct Args {
	/// minecraft version to upscale ex: 1.21.8, 25w37a, 25w14craftmine, 26.1-snapshot-4, 26.1
	version: String,
	/// automatically publish to modrinth (requires MODRINTH_KEY and MODRINTH_PROJECT_ID)
	#[arg(short, long)]
	publish: bool,
	/// send modrinth release to discord webhook (requires DISCORD_WEBHOOK)
	#[arg(short, long)]
	webhook: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse();

	let output_path = format!("./output/SnapshotXBR-{}.zip", args.version);
	if std::fs::exists(&output_path).expect("checking for existing output to work") {
		println!("{output_path} already exists, skipping client.jar fetching and packaging");
	} else {
		println!("fetching client.jar from mojang");
		let client_jar =
			mojang_api::get_client_files(&args.version).expect("jar file is valid zip archive");

		println!("processing into output zip");
		package_zip::package_zip(&output_path, client_jar);
		println!("processing done, output available at {output_path}");
	}

	if args.publish {
		dotenv::dotenv().ok();

		println!("publishing to modrinth");
		let release_url =
			modrinth_api::publish(&output_path, &args.version).expect("publishing to succeed");
		println!("{}", release_url);

		if args.webhook {
			println!("publishing to discord webhook");
			discord_webhook::post(&release_url).expect("webhook to succeed");
		}
	}

	Ok(())
}
