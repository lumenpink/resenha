use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use serde::Deserialize;

use crate::{assets, extractor, fetcher, formatter, ResenhaError};

#[derive(Debug, Deserialize)]
pub struct ResenhaQuery {
    pub url: Option<String>,
    pub format: Option<String>,
}

pub fn create_app() -> Router {
    Router::new()
        .route("/healthz", get(healthz_handler))
        .route("/", get(resenha_handler))
        .route("/resenha", get(resenha_handler))
        .route("/api/fuxicar", post(fuxicar_handler).get(fuxicar_handler))
}

async fn healthz_handler() -> &'static str {
    "ok"
}

async fn resenha_handler(
    headers: HeaderMap,
    Query(query): Query<ResenhaQuery>,
) -> Result<Response, AppError> {
    let target_url = match query.url {
        Some(ref u) if !u.trim().is_empty() => u.trim(),
        _ => {
            // Sem URL fornecida: renderiza a página inicial com o formulário
            return Ok(Html(formatter::render_home_page()).into_response());
        }
    };

    let raw_html = fetcher::fetch_html(target_url)
        .await
        .map_err(AppError::Internal)?;
    let mut article =
        extractor::extract_and_sanitize(&raw_html, target_url).map_err(AppError::Internal)?;

    let mut markdown = formatter::format_markdown(&article).map_err(AppError::Internal)?;

    // Download de assets caso DOWNLOAD_ASSETS não esteja explicitamente desativado
    let should_download_assets = std::env::var("DOWNLOAD_ASSETS")
        .map(|v| v != "false" && v != "0")
        .unwrap_or(true);

    if should_download_assets {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        if let Ok((rewritten_md, assets_list)) =
            assets::extract_and_download_assets(&client, target_url, &markdown).await
        {
            markdown = rewritten_md;
            article.assets = assets_list;
        }
    }

    let requested_format = query.format.as_deref().unwrap_or("");

    let wants_json = requested_format == "json"
        || headers
            .get(header::ACCEPT)
            .and_then(|h| h.to_str().ok())
            .map(|a| a.contains("application/json"))
            .unwrap_or(false);

    if wants_json {
        let json_body = serde_json::json!({
            "title": article.title,
            "source": article.source_url,
            "domain": article.domain,
            "author": article.author,
            "clipped_at": article.clipped_at,
            "markdown": markdown,
            "clean_html": article.clean_html,
            "assets": article.assets,
        });
        return Ok(Json(json_body).into_response());
    }

    let wants_markdown = requested_format == "md"
        || headers
            .get(header::ACCEPT)
            .and_then(|h| h.to_str().ok())
            .map(|a| a.contains("text/markdown") || a.contains("text/x-markdown"))
            .unwrap_or(false);

    if wants_markdown {
        return Ok((
            [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
            markdown,
        )
            .into_response());
    }

    // Padrão de navegação: renderiza a versão de leitura tipográfica compilada do Markdown
    let fuxico_api_url = std::env::var("FUXICO_API_URL").ok();
    let reading_page =
        formatter::render_reading_page(&article, &markdown, fuxico_api_url.as_deref());
    Ok((
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        reading_page,
    )
        .into_response())
}

async fn fuxicar_handler(Query(query): Query<ResenhaQuery>) -> Result<Response, AppError> {
    let target_url = query
        .url
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())
        .ok_or_else(|| AppError::BadRequest("Parâmetro 'url' obrigatório".to_string()))?;

    let fuxico_api_url = std::env::var("FUXICO_API_URL").map_err(|_| {
        AppError::BadRequest("Variável de ambiente FUXICO_API_URL não configurada".to_string())
    })?;

    let raw_html = fetcher::fetch_html(target_url)
        .await
        .map_err(AppError::Internal)?;
    let mut article =
        extractor::extract_and_sanitize(&raw_html, target_url).map_err(AppError::Internal)?;
    let mut markdown = formatter::format_markdown(&article).map_err(AppError::Internal)?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_default();

    if let Ok((rewritten_md, assets_list)) =
        assets::extract_and_download_assets(&client, target_url, &markdown).await
    {
        markdown = rewritten_md;
        article.assets = assets_list;
    }

    let safe_title: String = article
        .title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let filename = format!("{safe_title}.md");

    let mut form = reqwest::multipart::Form::new().text("render", "doc").part(
        "file",
        reqwest::multipart::Part::text(markdown).file_name(filename),
    );

    for asset in article.assets {
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&asset.data_base64) {
            let part = reqwest::multipart::Part::bytes(bytes)
                .file_name(asset.filename.clone())
                .mime_str(&asset.mime_type)
                .unwrap_or_else(|_| {
                    reqwest::multipart::Part::bytes(Vec::new()).file_name(asset.filename)
                });
            form = form.part("assets[]", part);
        }
    }

    let mut req = client.post(&fuxico_api_url).multipart(form);
    if let Ok(token) = std::env::var("FUXICO_AUTH_TOKEN") {
        if !token.trim().is_empty() {
            req = req.bearer_auth(token.trim());
        }
    }

    let resp = req
        .send()
        .await
        .map_err(|e| AppError::Internal(ResenhaError::Network(e)))?;
    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        return Ok((
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!("Fuxico retornou HTTP {status}: {body_text}")
            })),
        )
            .into_response());
    }

    let short_url = body_text.trim().to_string();
    Ok(Json(serde_json::json!({ "short_url": short_url })).into_response())
}

pub enum AppError {
    BadRequest(String),
    Internal(ResenhaError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            AppError::Internal(err) => {
                let status = match err {
                    ResenhaError::Network(_) => StatusCode::BAD_GATEWAY,
                    ResenhaError::InvalidUrl(_) => StatusCode::BAD_REQUEST,
                    _ => StatusCode::UNPROCESSABLE_ENTITY,
                };
                (status, format!("Erro no processamento: {err}")).into_response()
            }
        }
    }
}
