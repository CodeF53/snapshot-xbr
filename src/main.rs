use clap::Parser;

mod mojang_api;
mod package_zip;
mod process;

#[derive(clap::Parser)]
#[command(arg_required_else_help(true))]
struct Args {
	/// minecraft version to upscale ex: 1.21.8, 25w37a, 25w14craftmine, 26.1-snapshot-4, 26.1
	version: String,
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


	Ok(())
}
