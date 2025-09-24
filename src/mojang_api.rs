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

pub async fn get_client_jar(version: &str) -> Result<bytes::Bytes, reqwest::Error> {
	let client = reqwest::Client::new();
	let manifest_response: VersionManifest = client
		.get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
		.send()
		.await?
		.json()
		.await?;

	let version_url = manifest_response
		.versions
		.into_iter()
		.find(|v| v.id == version)
		.expect("Version not found in version_manifest.json")
		.url;
	let version_response: serde_json::Value = client.get(version_url).send().await?.json().await?;

	let jar_url = version_response
		.pointer("/downloads/client/url")
		.expect("mojang changed the shape of their data i think")
		.as_str()
		.expect("url key that doesn't point to a string?!");

	let jar = client.get(jar_url).send().await?.bytes().await?;
	Ok(jar)
}
