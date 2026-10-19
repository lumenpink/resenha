use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::{process_url, ResenhaError};

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
    let target_url = query.url.ok_or(AppError::BadRequest(
        "Parâmetro 'url' obrigatório".to_string(),
    ))?;

    if target_url.trim().is_empty() {
        return Err(AppError::BadRequest("URL não pode ser vazia".to_string()));
    }

    let markdown = process_url(&target_url).await.map_err(AppError::Internal)?;

    let wants_json = query.format.as_deref() == Some("json")
        || headers
            .get(header::ACCEPT)
            .and_then(|h| h.to_str().ok())
            .map(|a| a.contains("application/json"))
            .unwrap_or(false);

    if wants_json {
        let json_body = serde_json::json!({
            "url": target_url,
            "markdown": markdown,
        });
        Ok(Json(json_body).into_response())
    } else {
        Ok((
            [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
            markdown,
        )
            .into_response())
    }
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
