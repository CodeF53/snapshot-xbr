use serde::Deserialize;

#[derive(Deserialize)]
struct Version {
	id: String,
	url: reqwest::Url,
}

#[derive(Deserialize)]
struct VersionManifest {
	versions: Vec<Version>,
}

pub fn get_client_files(version: &str) -> Result<zip::ZipArchive<std::io::Cursor<bytes::Bytes>>, Box<dyn std::error::Error>> {
	let client = reqwest::blocking::Client::new();
	let manifest_response: VersionManifest = client
		.get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
		.send()?
		.json()?;

	let version_url = manifest_response
		.versions
		.into_iter()
		.find(|v| v.id == version)
		.expect("Version not found in version_manifest.json")
		.url;
	let version_response: serde_json::Value = client.get(version_url).send()?.json()?;

	let client_jar_url = version_response
		.pointer("/downloads/client/url")
		.expect("mojang changed the shape of their data i think")
		.as_str()
		.expect("url key that doesn't point to a string?!");

	let client_jar = client.get(client_jar_url).send()?.bytes()?;

	Ok(zip::ZipArchive::new(std::io::Cursor::new(client_jar))?)
}
