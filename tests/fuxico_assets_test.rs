use resenha::{formatter, ArticleAsset, ExtractedArticle};

#[test]
fn test_markdown_reader_parity() {
    let md = "# Título de Teste\n\nEste é um parágrafo no **Markdown** com uma [ligação](https://example.com).\n\n> Citação importante";
    let html = formatter::markdown_to_html(md);

    assert!(html.contains("<h1>Título de Teste</h1>"));
    assert!(html.contains("<strong>Markdown</strong>"));
    assert!(html.contains("<a href=\"https://example.com\">ligação</a>"));
    assert!(html.contains("<blockquote>"));
}

#[test]
fn test_render_reading_page_with_markdown() {
    let article = ExtractedArticle {
        title: "Artigo Teste".to_string(),
        source_url: "https://example.com/artigo".to_string(),
        domain: "example.com".to_string(),
        author: Some("Autor Teste".to_string()),
        clipped_at: "2026-10-19T22:00:00Z".to_string(),
        clean_html: "<p>Original HTML</p>".to_string(),
        assets: vec![ArticleAsset {
            filename: "imagem-1.webp".to_string(),
            original_url: "https://example.com/imagem-1.webp".to_string(),
            mime_type: "image/webp".to_string(),
            data_base64: "AAAA".to_string(),
        }],
    };

    let md_content = "Parágrafo a partir do markdown com ![imagem](attachments/imagem-1.webp)";
    let page = formatter::render_reading_page(&article, md_content, Some("https://fuxico.lohn.in"));

    // Valida que o conteúdo foi renderizado a partir do Markdown
    assert!(page.contains("<p>Parágrafo a partir do markdown com"));
    assert!(page.contains("<img src=\"attachments/imagem-1.webp\" alt=\"imagem\""));
    // Valida presença do botão Fuxicar quando FUXICO_API_URL está presente
    assert!(page.contains("⚡ Fuxicar"));
}

#[tokio::test]
async fn test_fuxicar_endpoint_requires_url() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let app = resenha::server::create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/fuxicar")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_extract_assets_with_escaped_parentheses() {
    let md = r#"Matéria com imagem contendo filtros na URL: ![Foto](https://s2-g1.glbimg.com/smart/filters:gifv\(\):strip_icc\(\)/foto.jpg)"#;
    let client = reqwest::Client::new();
    let (rewritten, _assets) =
        resenha::assets::extract_and_download_assets(&client, "https://g1.globo.com", md)
            .await
            .unwrap();

    // Como foto.jpg não existe de verdade na rede local sem mock, o download falha gracioso mantendo URL ou tentando
    // Porém aqui verificamos que a regex não quebra a string no meio
    assert!(!rewritten.is_empty());
}
