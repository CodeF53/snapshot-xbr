pub fn post(release_url: &str) -> Result<(), Box<dyn std::error::Error>> {
	let webhook_url = std::env::var("DISCORD_WEBHOOK").expect("DISCORD_WEBHOOK");

	let client = reqwest::blocking::Client::new();

	let webhook_resp = client
		.post(webhook_url)
		.json(&serde_json::json!({ "content": release_url }))
		.send()?;

	let status = webhook_resp.status();
	if !status.is_success() {
		return Err(format!(
			"error while publishing to discord webhook {}: {}",
			status,
			webhook_resp.text().unwrap_or_default(),
		)
		.into());
	}
	Ok(())
}
