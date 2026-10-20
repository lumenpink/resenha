use std::io::Cursor;
use url::Url;

use crate::{ExtractedArticle, ResenhaError};

pub fn extract_and_sanitize(html: &str, url_str: &str) -> Result<ExtractedArticle, ResenhaError> {
    let parsed_url = Url::parse(url_str)?;
    let domain = parsed_url.host_str().unwrap_or("").to_string();

    let mut cursor = Cursor::new(html.as_bytes());
    let product = readability::extractor::extract(&mut cursor, &parsed_url)
        .map_err(|e| ResenhaError::Extraction(format!("Erro no algoritmo Readability: {e}")))?;

    // Configuração estrita de sanitização via Ammonia (Anti-XSS, sem iframes, sem scripts)
    let clean_html = ammonia::Builder::new()
        .tags(
            [
                "h1",
                "h2",
                "h3",
                "h4",
                "h5",
                "h6",
                "p",
                "blockquote",
                "pre",
                "code",
                "em",
                "strong",
                "b",
                "i",
                "u",
                "s",
                "del",
                "ul",
                "ol",
                "li",
                "a",
                "img",
                "table",
                "thead",
                "tbody",
                "tr",
                "th",
                "td",
                "hr",
                "br",
            ]
            .into_iter()
            .collect(),
        )
        .clean(&product.content)
        .to_string();

    let title = if product.title.trim().is_empty() {
        "Sem título".to_string()
    } else {
        product.title.trim().to_string()
    };

    Ok(ExtractedArticle {
        title,
        source_url: url_str.to_string(),
        domain,
        author: None,
        clipped_at: chrono::Utc::now().to_rfc3339(),
        clean_html,
        assets: Vec::new(),
    })
}
