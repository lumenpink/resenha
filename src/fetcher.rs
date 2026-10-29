use crate::ResenhaError;
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, UPGRADE_INSECURE_REQUESTS, USER_AGENT,
};
use std::time::Duration;

const BROWSER_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0";

fn build_browser_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(BROWSER_USER_AGENT));
    headers.insert(
        ACCEPT,
        HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        ),
    );
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("pt-BR,pt;q=0.9,en-US;q=0.8,en;q=0.7"),
    );
    headers.insert(UPGRADE_INSECURE_REQUESTS, HeaderValue::from_static("1"));
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-dest"),
        HeaderValue::from_static("document"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-mode"),
        HeaderValue::from_static("navigate"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-site"),
        HeaderValue::from_static("none"),
    );
    headers
}

fn get_timeout() -> Duration {
    if let Ok(secs_str) = std::env::var("RESENHA_TIMEOUT_SECS") {
        if let Ok(secs) = secs_str.parse::<u64>() {
            return Duration::from_secs(secs);
        }
    }
    Duration::from_secs(15)
}

fn get_fallback_url(url: &str) -> Option<String> {
    let enabled = std::env::var("RESENHA_ENABLE_FALLBACK")
        .map(|v| v != "0" && v != "false" && v != "no")
        .unwrap_or(true);

    if !enabled {
        return None;
    }

    let pattern = std::env::var("RESENHA_FALLBACK_URL")
        .unwrap_or_else(|_| "https://r.jina.ai/{url}".to_string());

    let target = if pattern.contains("{url}") {
        pattern.replace("{url}", url)
    } else if pattern.contains("{}") {
        pattern.replace("{}", url)
    } else {
        format!("{pattern}{url}")
    };

    Some(target)
}

async fn fetch_via_fallback(
    client: &reqwest::Client,
    fallback_url: &str,
) -> Result<String, ResenhaError> {
    let req = client
        .get(fallback_url)
        .header("X-Return-Format", "html")
        .timeout(get_timeout());

    let resp = req.send().await?;
    let body = resp.error_for_status()?.text().await?;
    Ok(body)
}

pub async fn fetch_html(url: &str) -> Result<String, ResenhaError> {
    let client = reqwest::Client::builder()
        .timeout(get_timeout())
        .default_headers(build_browser_headers())
        .build()?;

    let response = client.get(url).send().await?;

    if response.status() == reqwest::StatusCode::FORBIDDEN
        || response.status() == reqwest::StatusCode::UNAUTHORIZED
    {
        if let Some(fallback_url) = get_fallback_url(url) {
            if let Ok(fallback_body) = fetch_via_fallback(&client, &fallback_url).await {
                return Ok(fallback_body);
            }
        }
    }

    let body = response.error_for_status()?.text().await?;
    Ok(body)
}
