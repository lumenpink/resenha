use crate::ResenhaError;
use std::time::Duration;

const USER_AGENT: &str =
    "Mozilla/5.0 (compatible; Resenha/0.1; +https://codeberg.org/lumenpink/resenha)";

pub async fn fetch_html(url: &str) -> Result<String, ResenhaError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(USER_AGENT)
        .build()?;

    let response = client.get(url).send().await?;

    let body = response.error_for_status()?.text().await?;
    Ok(body)
}
