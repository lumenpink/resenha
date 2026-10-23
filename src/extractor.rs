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

    // 3. Aplanar figuras duplamente aninhadas (<figure><figure><img ...></figure></figure>)
    if let Ok(nested_fig_re) = regex::Regex::new(
        r#"(?is)<figure[^>]*>\s*(?:<!--.*?-->\s*)?<figure[^>]*>\s*(<img[^>]+>).*?<figcaption[^>]*>(.*?)</figcaption>\s*</figure>\s*(?:<figcaption[^>]*>.*?</figcaption>\s*)?</figure>"#,
    ) {
        processed = nested_fig_re
            .replace_all(&processed, "<figure>$1<figcaption>$2</figcaption></figure>")
            .to_string();
    }

    // 4. Desenvelopar divs com 'content-media' que envolvem <figure>
    // O Readability avalia <div> com is_useless e descarta divs contendo 'media' onde img_count > para_count.
    // Ao desenvelopar a div e manter apenas o <figure>, o Readability preserva a imagem perfeitamente.
    if let Ok(unwrap_media_re) = regex::Regex::new(
        r#"(?is)<div\b[^>]*class\s*=\s*["'][^"']*\b(?:content-media|content-figure)\b[^"']*["'][^>]*>\s*(<figure\b.*?</figure>)\s*</div>"#,
    ) {
        processed = unwrap_media_re.replace_all(&processed, "$1").to_string();
    }

    // 5. Desenvelopar divs externas que envolvem exclusivamente um <figure>
    // Portais como G1 encapsulam cada bloco em <div id="chunk-...">. O Readability considera
    // qualquer <div> contendo imagens e sem parágrafos como lixo (scorer::is_useless).
    // Ao desenvelopar a div e expor o <figure>, o Readability preserva a imagem no artigo.
    if let Ok(unwrap_div_fig_re) =
        regex::Regex::new(r#"(?is)<div\b[^>]*>\s*(<figure\b.*?</figure>)\s*</div>"#)
    {
        for _ in 0..5 {
            let replaced = unwrap_div_fig_re.replace_all(&processed, "$1").to_string();
            if replaced == processed {
                break;
            }
            processed = replaced;
        }
    }

    // 6. Neutralizar divs de paywall do G1/Globo ('wall', 'protected-content')
    // O G1 isola o texto pós-primeiro-parágrafo dentro de <div class="wall protected-content">.
    // Convertendo a classe para 'article-body', o Readability reconhece como container positivo.
    if let Ok(wall_re) = regex::Regex::new(
        r#"(?i)<div\b([^>]*)class\s*=\s*["'][^"']*\b(?:wall|protected-content)\b[^"']*["']"#,
    ) {
        processed = wall_re
            .replace_all(&processed, r#"<div$1class="article-body""#)
            .to_string();
    }

    // 7. Desenvelopar divs de coluna de texto ('content-text') e blocos ('chunk-*') do G1
    // O G1 divide cada parágrafo em seu próprio chunk (<div id="chunk-XXXX"><div class="mc-column content-text active-extra-styles"><p>...</p></div></div>).
    // Como o Readability pontua apenas parent e parent.parent, divs isoladas não acumulam a pontuação do artigo inteiro.
    // Desenvelopar esses wrappers expõe os <p> diretamente sob o container principal.
    if let Ok(unwrap_content_text_re) = regex::Regex::new(
        r#"(?is)<div\b[^>]*class\s*=\s*["'][^"']*\bcontent-text\b[^"']*["'][^>]*>\s*(<p\b.*?</p>)\s*</div>"#,
    ) {
        for _ in 0..5 {
            let replaced = unwrap_content_text_re
                .replace_all(&processed, "$1")
                .to_string();
            if replaced == processed {
                break;
            }
            processed = replaced;
        }
    }
    if let Ok(unwrap_chunk_re) = regex::Regex::new(
        r#"(?is)<div\b[^>]*id\s*=\s*["']chunk-[^"']*["'][^>]*>\s*(<(?:p|figure)\b.*?</(?:p|figure)>)\s*</div>"#,
    ) {
        for _ in 0..5 {
            let replaced = unwrap_chunk_re.replace_all(&processed, "$1").to_string();
            if replaced == processed {
                break;
            }
            processed = replaced;
        }
    }

    // 8. Desarmar classes remanescentes com 'media'
    if let Ok(media_class_re) = regex::Regex::new(r#"(?i)\bcontent-media\b"#) {
        processed = media_class_re
            .replace_all(&processed, "content-figure")
            .to_string();
    }

    // 7. Promover figuras de destaque externas para dentro do bloco do artigo (antes do 1º parágrafo)
    // Portais como O Globo posicionam a foto principal no <section class="content--header"> fora de <article> ou <div class="no-paywall">.
    // Inserir a figura logo antes do primeiro parágrafo garante que o Readability a preserve no corpo extraído.
    if let (Ok(top_fig_re), Ok(p_re)) = (
        regex::Regex::new(
            r#"(?is)<figure\b[^>]*class\s*=\s*["'][^"']*\b(?:content-featured-figure|content-top-image|featured-figure|article-lead-image)\b[^"']*["'][^>]*>.*?</figure>"#,
        ),
        regex::Regex::new(r#"(?i)<p\b[^>]*>"#),
    ) {
        if let Some(m_fig) = top_fig_re.find(&processed) {
            if let Some(_m_p) = p_re.find_at(&processed, m_fig.end()) {
                let fig_html = m_fig.as_str().to_string();
                let without_fig = format!(
                    "{}{}",
                    &processed[..m_fig.start()],
                    &processed[m_fig.end()..]
                );
                if let Some(m_p2) = p_re.find_at(&without_fig, m_fig.start()) {
                    processed = format!(
                        "{}{}\n{}",
                        &without_fig[..m_p2.start()],
                        fig_html,
                        &without_fig[m_p2.start()..]
                    );
                }
            }
        }
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
