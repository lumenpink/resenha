use std::io::Cursor;
use url::Url;

use crate::{ExtractedArticle, ResenhaError};

pub fn preprocess_html(html: &str) -> String {
    // 1. Converter <amp-img ...> para <img ...> e remover </amp-img>
    let mut processed = html.replace("</amp-img>", "");
    if let Ok(amp_re) = regex::Regex::new(r"(?i)<amp-img\b([^>]*)>") {
        processed = amp_re.replace_all(&processed, "<img$1>").to_string();
    }

    // 2. Normalizar atributos lazy-load em tags <img>
    if let Ok(img_re) = regex::Regex::new(r"(?i)<img\b([^>]*)/?>") {
        processed = img_re
            .replace_all(&processed, |caps: &regex::Captures| {
                let attrs = &caps[1];
                let rewritten_attrs = normalize_img_attributes(attrs);
                format!("<img {rewritten_attrs}>")
            })
            .to_string();
    }

    processed
}

fn normalize_img_attributes(attrs: &str) -> String {
    let src_re = regex::Regex::new(r#"(?i)\bsrc\s*=\s*["']([^"']+)["']"#).ok();
    let data_src_re = regex::Regex::new(
        r#"(?i)\b(?:data-src|data-original|data-lazy-src|data-actualsrc)\s*=\s*["']([^"']+)["']"#,
    )
    .ok();

    let current_src = src_re
        .as_ref()
        .and_then(|r| r.captures(attrs))
        .and_then(|c| c.get(1))
        .map(|m| m.as_str());

    let is_placeholder = match current_src {
        None => true,
        Some(s) => {
            let s_trim = s.trim();
            s_trim.is_empty()
                || s_trim.starts_with("data:image")
                || s_trim == "#"
                || s_trim == "about:blank"
        }
    };

    if is_placeholder {
        if let Some(caps) = data_src_re.as_ref().and_then(|r| r.captures(attrs)) {
            if let Some(real_url) = caps.get(1).map(|m| m.as_str()) {
                let mut new_attrs = attrs.to_string();
                if let Some(r) = src_re.as_ref() {
                    new_attrs = r.replace(&new_attrs, "").to_string();
                }
                return format!(r#"src="{real_url}" {new_attrs}"#);
            }
        }
    }

    attrs.to_string()
}

pub fn extract_and_sanitize(html: &str, url_str: &str) -> Result<ExtractedArticle, ResenhaError> {
    let parsed_url = Url::parse(url_str)?;
    let domain = parsed_url.host_str().unwrap_or("").to_string();

    let preprocessed_html = preprocess_html(html);

    let mut cursor = Cursor::new(preprocessed_html.as_bytes());
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
                "figure",
                "figcaption",
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
