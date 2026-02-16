use reqwest::{
	blocking::{Client, Response, multipart},
	header::{AUTHORIZATION, HeaderMap, HeaderValue},
};

pub fn publish(file_path: &str, version: &str) -> Result<String, Box<dyn std::error::Error>> {
	let key = std::env::var("MODRINTH_KEY").expect("MODRINTH_KEY");
	let project_id = std::env::var("MODRINTH_PROJECT_ID").expect("MODRINTH_PROJECT_ID");

	let mut headers = HeaderMap::new();
	headers.insert(AUTHORIZATION, HeaderValue::from_str(&key).unwrap());

	let client = Client::builder()
		.default_headers(headers)
		.user_agent("CodeF53/snapshot-xbr (fseusb@gmail.com)")
		.build()?;

	if has_existing_versions(&client, &project_id, version)? {
		return Err(format!("There is already a release for {version}").into());
	}

	const FILE_PART_NAME: &str = "pack";
	let data = serde_json::json!({
		"name": version,
		"version_number": version,
		"changelog": format!(
			"https://www.minecraft.net/en-us/article/minecraft-{}",
			version.replace('.', "-")
		),
		"dependencies": [],
		"game_versions": [version],
		"version_type": "release",
		"loaders": ["minecraft"],
		"featured": false,
		"status": "listed",
		"project_id": project_id,
		"file_parts": [FILE_PART_NAME],
	})
	.to_string();
	let form = multipart::Form::new()
		.text("data", data)
		.file(FILE_PART_NAME, file_path)?;

	let resp = client
		.post("https://api.modrinth.com/v2/version")
		.multipart(form)
		.send()?;
	if !resp.status().is_success() {
		return Err(handle_modrinth_error(resp));
	}

	Ok(format!(
		"https://modrinth.com/resourcepack/{project_id}/version/{version}"
	))
}

fn has_existing_versions(
	client: &Client,
	project_id: &str,
	version: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
	let resp = client
		.get(format!(
			"https://api.modrinth.com/v2/project/{project_id}/version"
		))
		.query(&[("game_versions", serde_json::json!([version]).to_string())])
		.send()?;
	if !resp.status().is_success() {
		return Err(handle_modrinth_error(resp));
	}
	Ok(!resp.json::<Vec<serde_json::Value>>()?.is_empty())
}

fn handle_modrinth_error(resp: Response) -> Box<dyn std::error::Error> {
	format!(
		"modrinth api error {}: {}",
		resp.status(),
		resp.text().unwrap_or_default()
	)
	.into()
}
