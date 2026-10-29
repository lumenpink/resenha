use axum::{
    extract::Request,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;

static ENV_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test]
async fn test_fetcher_fallback_when_origin_returns_403() {
    let _guard = ENV_MUTEX.lock().await;

    let fallback_counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = fallback_counter.clone();

    let mock_app = Router::new()
        .route(
            "/bloqueada",
            get(|| async { (StatusCode::FORBIDDEN, "Access Denied").into_response() }),
        )
        .route(
            "/fallback/{*rest}",
            get(move |req: Request| {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    let format_hdr = req
                        .headers()
                        .get("x-return-format")
                        .and_then(|v| v.to_str().ok());
                    assert_eq!(format_hdr, Some("html"));
                    (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                        "<html><body><article><h1>Artigo via Fallback</h1><p>Conteudo recuperado com sucesso.</p></article></body></html>",
                    )
                        .into_response()
                }
            }),
        );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, mock_app).await.unwrap();
    });

    let origin_url = format!("http://{addr}/bloqueada");
    let fallback_url_template = format!("http://{addr}/fallback/{{url}}");

    std::env::set_var("RESENHA_FALLBACK_URL", fallback_url_template);
    std::env::set_var("RESENHA_ENABLE_FALLBACK", "true");

    let result = resenha::fetcher::fetch_html(&origin_url).await;
    assert!(
        result.is_ok(),
        "Fetcher deve recuperar via fallback quando a origem der 403: {:?}",
        result.err()
    );
    let html = result.unwrap();
    assert!(html.contains("Artigo via Fallback"));
    assert_eq!(fallback_counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_fetcher_fails_when_fallback_disabled() {
    let _guard = ENV_MUTEX.lock().await;

    let mock_app = Router::new().route(
        "/bloqueada-sem-fallback",
        get(|| async { (StatusCode::FORBIDDEN, "Access Denied").into_response() }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, mock_app).await.unwrap();
    });

    let origin_url = format!("http://{addr}/bloqueada-sem-fallback");

    std::env::set_var("RESENHA_ENABLE_FALLBACK", "false");

    let result = resenha::fetcher::fetch_html(&origin_url).await;
    assert!(
        result.is_err(),
        "Fetcher deve falhar quando fallback estiver desativado e origem der 403"
    );
}

#[tokio::test]
async fn test_fetcher_origin_404_does_not_trigger_fallback() {
    let _guard = ENV_MUTEX.lock().await;

    let fallback_counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = fallback_counter.clone();

    let mock_app = Router::new()
        .route(
            "/nao-existe",
            get(|| async { (StatusCode::NOT_FOUND, "Not Found").into_response() }),
        )
        .route(
            "/fallback/{*rest}",
            get(move || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    (StatusCode::OK, "Nunca deve ser chamado").into_response()
                }
            }),
        );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, mock_app).await.unwrap();
    });

    let origin_url = format!("http://{addr}/nao-existe");
    let fallback_url_template = format!("http://{addr}/fallback/{{url}}");

    std::env::set_var("RESENHA_FALLBACK_URL", fallback_url_template);
    std::env::set_var("RESENHA_ENABLE_FALLBACK", "true");

    let result = resenha::fetcher::fetch_html(&origin_url).await;
    assert!(result.is_err(), "404 deve propagar erro sem fallback");
    assert_eq!(
        fallback_counter.load(Ordering::SeqCst),
        0,
        "Fallback não deve ser acionado para HTTP 404"
    );
}

#[tokio::test]
async fn test_fetcher_origin_403_and_fallback_fails() {
    let _guard = ENV_MUTEX.lock().await;

    let mock_app = Router::new()
        .route(
            "/bloqueada-dupla",
            get(|| async { (StatusCode::FORBIDDEN, "Access Denied").into_response() }),
        )
        .route(
            "/fallback-com-falha/{*rest}",
            get(|| async {
                (StatusCode::INTERNAL_SERVER_ERROR, "Fallback Server Error").into_response()
            }),
        );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, mock_app).await.unwrap();
    });

    let origin_url = format!("http://{addr}/bloqueada-dupla");
    let fallback_url_template = format!("http://{addr}/fallback-com-falha/{{url}}");

    std::env::set_var("RESENHA_FALLBACK_URL", fallback_url_template);
    std::env::set_var("RESENHA_ENABLE_FALLBACK", "true");

    let result = resenha::fetcher::fetch_html(&origin_url).await;
    assert!(
        result.is_err(),
        "Fetcher deve retornar erro quando tanto a origem quanto o fallback falharem"
    );
}
