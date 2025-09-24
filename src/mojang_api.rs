use futures::StreamExt;
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

#[derive(Deserialize)]
struct Version {
	id: String,
	url: reqwest::Url,
}

#[derive(Deserialize)]
struct VersionManifest {
	versions: Vec<Version>,
}

pub async fn get_client_files(version: &str) -> Result<(), Box<dyn std::error::Error>> {
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

	let client_jar_url = version_response
		.pointer("/downloads/client/url")
		.expect("mojang changed the shape of their data i think")
		.as_str()
		.expect("url key that doesn't point to a string?!");

	let client_mappings_url = version_response
		.pointer("/downloads/client_mappings/url")
		.expect("mojang changed the shape of their data i think")
		.as_str()
		.expect("url key that doesn't point to a string?!");

	if !std::fs::exists("./tmp/")? {
		std::fs::create_dir("./tmp")?;
	}

	tokio::try_join!(
		stream_files(&client, client_jar_url, "./tmp/client.jar"),
		stream_files(&client, client_mappings_url, "./tmp/client.txt"),
	)?;

	Ok(())
}

async fn stream_files(
	client: &reqwest::Client,
	url: impl reqwest::IntoUrl,
	path: impl AsRef<std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
	let response = client.get(url).send().await?;

	if !response.status().is_success() {
		return Err(response.status().canonical_reason().unwrap().into());
	};

	let mut file = tokio::fs::File::create(path).await?;

	let mut response_stream = response.bytes_stream();
	while let Some(chunk_result) = response_stream.next().await {
		let chunk = chunk_result?;
		file.write(&chunk).await?;
	}
	file.flush().await?;
	Ok(())
}
