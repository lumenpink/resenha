use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::{extractor, fetcher, formatter, ResenhaError};

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
    let article =
        extractor::extract_and_sanitize(&raw_html, target_url).map_err(AppError::Internal)?;

    let requested_format = query.format.as_deref().unwrap_or("");

    let wants_json = requested_format == "json"
        || headers
            .get(header::ACCEPT)
            .and_then(|h| h.to_str().ok())
            .map(|a| a.contains("application/json"))
            .unwrap_or(false);

    if wants_json {
        let markdown = formatter::format_markdown(&article).map_err(AppError::Internal)?;
        let json_body = serde_json::json!({
            "title": article.title,
            "source": article.source_url,
            "domain": article.domain,
            "author": article.author,
            "clipped_at": article.clipped_at,
            "markdown": markdown,
            "clean_html": article.clean_html,
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
        let markdown = formatter::format_markdown(&article).map_err(AppError::Internal)?;
        return Ok((
            [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
            markdown,
        )
            .into_response());
    }

    // Padrão de navegação: renderiza a versão de leitura tipográfica soberana
    let reading_page = formatter::render_reading_page(&article);
    Ok((
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        reading_page,
    )
        .into_response())
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
