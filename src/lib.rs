pub mod extractor;
pub mod fetcher;
pub mod formatter;
pub mod server;

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
}

pub async fn process_url(url: &str) -> Result<String, ResenhaError> {
    todo!("implementar process_url")
}

pub fn process_html(html: &str, url: &str) -> Result<String, ResenhaError> {
    todo!("implementar process_html")
}
