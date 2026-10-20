use base64::Engine;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

use crate::ResenhaError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleAsset {
    pub filename: String,
    pub original_url: String,
    pub mime_type: String,
    pub data_base64: String,
}

pub async fn extract_and_download_assets(
    client: &reqwest::Client,
    base_url_str: &str,
    markdown: &str,
) -> Result<(String, Vec<ArticleAsset>), ResenhaError> {
    let base_url = Url::parse(base_url_str)?;
    let mut assets = Vec::new();
    let mut rewritten_markdown = markdown.to_string();

    // Regex para capturar ![alt](url), suportando parênteses escapados dentro da URL (ex.: filters:gifv\(\):strip_icc\(\))
    let re = match regex::Regex::new(
        r"!\[([^\]]*)\]\((https?://(?:\\\(|\\\)|[^)\s])+)(?:\s+[^)]+)?\)",
    ) {
        Ok(r) => r,
        Err(_) => return Ok((rewritten_markdown, assets)),
    };

    let mut asset_counter = 1;
    let mut seen_urls = std::collections::HashMap::new();

    for caps in re.captures_iter(markdown) {
        let full_match = match caps.get(0) {
            Some(m) => m.as_str().to_string(),
            None => continue,
        };
        let alt_text = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let raw_url = match caps.get(2) {
            Some(m) => m.as_str(),
            None => continue,
        };
        let original_url = raw_url.replace(r"\(", "(").replace(r"\)", ")");

        let resolved_url = match base_url.join(&original_url) {
            Ok(u) => u.to_string(),
            Err(_) => original_url.to_string(),
        };

        if let Some(existing_filename) = seen_urls.get(&resolved_url) {
            let replacement = format!("![{alt_text}](attachments/{existing_filename})");
            rewritten_markdown = rewritten_markdown.replace(&full_match, &replacement);
            continue;
        }

        // Tentar baixar imagem com timeout estrito de 5s
        let resp = match client
            .get(&resolved_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => r,
            _ => continue, // se falhar, mantém a URL original
        };

        let content_type = resp
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();

        let ext = match content_type.as_str() {
            "image/png" => "png",
            "image/webp" => "webp",
            "image/gif" => "gif",
            "image/svg+xml" => "svg",
            "image/avif" => "avif",
            _ => "jpg",
        };

        let bytes = match resp.bytes().await {
            Ok(b) if !b.is_empty() && b.len() <= 10 * 1024 * 1024 => b, // max 10MB
            _ => continue,
        };

        let filename = format!("imagem-{asset_counter}.{ext}");
        asset_counter += 1;

        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

        assets.push(ArticleAsset {
            filename: filename.clone(),
            original_url: resolved_url.clone(),
            mime_type: content_type,
            data_base64: b64,
        });

        seen_urls.insert(resolved_url, filename.clone());

        let replacement = format!("![{alt_text}](attachments/{filename})");
        rewritten_markdown = rewritten_markdown.replace(&full_match, &replacement);
    }

    Ok((rewritten_markdown, assets))
}

pub fn create_editorial_zip(
    markdown: &str,
    assets: &[ArticleAsset],
) -> Result<Vec<u8>, ResenhaError> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buffer);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // 1. main.md na raiz do ZIP
        zip.start_file("main.md", options)
            .map_err(|e| ResenhaError::Asset(format!("Erro ao criar main.md no ZIP: {e}")))?;
        zip.write_all(markdown.as_bytes())
            .map_err(|e| ResenhaError::Asset(format!("Erro ao escrever main.md: {e}")))?;

        // 2. attachments/<filename>
        for asset in assets {
            let path = format!("attachments/{}", asset.filename);
            zip.start_file(&path, options)
                .map_err(|e| ResenhaError::Asset(format!("Erro ao criar {path} no ZIP: {e}")))?;
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&asset.data_base64)
            {
                zip.write_all(&bytes)
                    .map_err(|e| ResenhaError::Asset(format!("Erro ao escrever {path}: {e}")))?;
            }
        }

        zip.finish()
            .map_err(|e| ResenhaError::Asset(format!("Erro ao finalizar ZIP: {e}")))?;
    }

    Ok(buffer.into_inner())
}
