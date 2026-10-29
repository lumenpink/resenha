use axum::http::StatusCode;
use resenha::formatter;

#[test]
fn test_render_error_page_contains_styling_and_status() {
    let html = formatter::render_error_page(
        StatusCode::NOT_FOUND,
        "Artigo Não Encontrado",
        "Não foi possível encontrar a página requisitada na fonte original.",
        Some("HTTP status client error (404 Not Found)"),
        false, // permanente
    );

    assert!(html.contains("404"), "Deve conter código de status");
    assert!(
        html.contains("Artigo Não Encontrado"),
        "Deve conter o título do erro"
    );
    assert!(
        html.contains("Não foi possível encontrar a página"),
        "Deve conter a mensagem amigável"
    );
    assert!(
        html.contains("HTTP status client error"),
        "Deve conter os detalhes técnicos"
    );
    assert!(html.contains("✂ resenha"), "Deve conter o logotipo");
    assert!(
        html.contains("Voltar à página inicial"),
        "Deve conter botão/link de retorno"
    );
    assert!(
        html.contains("Tentar novamente"),
        "Deve conter botão de tentar novamente"
    );
    assert!(
        html.contains("location.reload()"),
        "Deve conter ação de reload na tentativa"
    );
    assert!(
        html.contains("Bloqueio Definitivo") || html.contains("Erro Permanente"),
        "Deve identificar erro permanente"
    );
}

#[test]
fn test_render_error_page_transient_error() {
    let html = formatter::render_error_page(
        StatusCode::GATEWAY_TIMEOUT,
        "Tempo Limite Esgotado",
        "A conexão com o servidor de origem expirou antes de receber a resposta.",
        Some("operation timed out"),
        true, // temporário / transitório
    );

    assert!(html.contains("504"), "Deve conter código de status");
    assert!(
        html.contains("Instabilidade Temporária"),
        "Deve identificar erro transitório"
    );
    assert!(
        html.contains("Tentar novamente"),
        "Deve conter botão de tentar novamente"
    );
}

#[tokio::test]
async fn test_invalid_path_returns_404_html() {
    use axum::body::Body;
    use axum::http::{header, Request};
    use tower::ServiceExt;

    let app = resenha::server::create_app();

    // Requisitar um caminho inválido sem formato de URL, como attachments/imagem-1.jpg
    let response = app
        .oneshot(
            Request::builder()
                .uri("/attachments/imagem-1.jpg")
                .header(header::ACCEPT, "text/html")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&bytes);
    assert!(body_str.contains("404"));
    assert!(body_str.contains("✂ resenha"));
    assert!(body_str.contains("Tentar novamente"));
}
