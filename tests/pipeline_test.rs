use resenha::{extractor, formatter, server, ExtractedArticle};

const FIXTURE_HTML: &str = include_str!("fixtures/noisy_article.html");
const TARGET_URL: &str = "https://banca.2lp.in/artigo-soberania";

#[test]
fn test_extractor_sanitizes_scripts_and_iframes() {
    let result = extractor::extract_and_sanitize(FIXTURE_HTML, TARGET_URL)
        .expect("O extrator deve processar o HTML com sucesso");

    // Validação de Título e Conteúdo
    assert!(
        result.title.contains("A Soberania da Leitura"),
        "Título extraído incorreto: {}",
        result.title
    );
    assert!(
        result.clean_html.contains("verdadeira elegância"),
        "O corpo do texto não contém o artigo esperado"
    );

    // Validação de Higiene Anti-XSS e Anti-Rastreamento
    assert!(
        !result.clean_html.to_lowercase().contains("<script"),
        "HTML limpo não deve conter tags <script>"
    );
    assert!(
        !result.clean_html.to_lowercase().contains("<iframe"),
        "HTML limpo não deve conter tags <iframe>"
    );
    assert!(
        !result.clean_html.to_lowercase().contains("onclick"),
        "HTML limpo não deve conter atributos onclick"
    );
    assert!(
        !result.clean_html.contains("Compre agora antes que acabe"),
        "HTML limpo não deve conter o banner publicitário"
    );
}

#[test]
fn test_formatter_generates_markdown_with_frontmatter_and_original_link() {
    let extracted = ExtractedArticle {
        title: "A Soberania da Leitura na Borda Digital".to_string(),
        source_url: TARGET_URL.to_string(),
        domain: "banca.2lp.in".to_string(),
        author: Some("Lumen Pink".to_string()),
        clipped_at: "2026-10-19T14:45:00Z".to_string(),
        clean_html: "<h2>Pilares</h2><p>Texto de teste.</p><blockquote>Citação inspiradora</blockquote><p><a href=\"https://fuxico.lumen.pink\">Fuxico</a></p>".to_string(),
    };

    let md = formatter::format_markdown(&extracted)
        .expect("Formatação para Markdown deve ter sucesso");

    // Validação estrita do YAML Frontmatter
    assert!(md.starts_with("---\n"), "Markdown deve começar com YAML Frontmatter");
    assert!(md.contains("source: \"https://banca.2lp.in/artigo-soberania\""), "Frontmatter deve conter source exato");
    assert!(md.contains("domain: \"banca.2lp.in\""), "Frontmatter deve conter domain");
    assert!(md.contains("generator: \"resenha\""), "Frontmatter deve conter generator");
    assert!(md.contains("author: \"Lumen Pink\""), "Frontmatter deve conter autor");

    // Validação obrigatória da Badge de Fonte Original visível no topo
    assert!(
        md.contains("> ↗ **Fonte original:** [banca.2lp.in](https://banca.2lp.in/artigo-soberania)"),
        "Markdown deve conter a citação destacada da fonte original no topo"
    );

    // Validação da semântica Markdown
    assert!(md.contains("## Pilares"), "Cabeçalhos h2 devem ser convertidos");
    assert!(md.contains("> Citação inspiradora"), "Blockquotes devem ser preservados");
    assert!(md.contains("[Fuxico](https://fuxico.lumen.pink)"), "Links devem ser preservados");
}

#[tokio::test]
async fn test_server_healthz_endpoint() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let app = server::create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"ok");
}
