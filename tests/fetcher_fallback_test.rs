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

#[tokio::test]
async fn test_fetcher_fallback_when_origin_returns_403() {
    let fallback_counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = fallback_counter.clone();

    // Cria um servidor mock que retorna 403 na rota /bloqueada
    // e retorna HTML na rota /fallback/{rest}
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

    // Configura as variáveis de ambiente para o teste
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

    // Desativa explicitamente o fallback
    std::env::set_var("RESENHA_ENABLE_FALLBACK", "false");

    let result = resenha::fetcher::fetch_html(&origin_url).await;
    assert!(
        result.is_err(),
        "Fetcher deve falhar quando fallback estiver desativado e origem der 403"
    );
}
