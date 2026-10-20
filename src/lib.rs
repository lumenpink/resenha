pub mod assets;
pub mod extractor;
pub mod fetcher;
pub mod formatter;
pub mod server;

pub use assets::ArticleAsset;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResenhaError {
    #[error("Falha de rede ao buscar URL: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Falha na extração do conteúdo: {0}")]
    Extraction(String),

    #[error("Falha na formatação de saída: {0}")]
    Formatting(String),

    #[error("Falha no processamento de assets: {0}")]
    Asset(String),

    #[error("URL inválida: {0}")]
    InvalidUrl(#[from] url::ParseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedArticle {
    pub title: String,
    pub source_url: String,
    pub domain: String,
    pub author: Option<String>,
    pub clipped_at: String,
    pub clean_html: String,
    #[serde(default)]
    pub assets: Vec<ArticleAsset>,
}

pub async fn process_url(url: &str) -> Result<String, ResenhaError> {
    let raw_html = fetcher::fetch_html(url).await?;
    process_html(&raw_html, url)
}

pub fn process_html(html: &str, url: &str) -> Result<String, ResenhaError> {
    let article = extractor::extract_and_sanitize(html, url)?;
    formatter::format_markdown(&article)
}
