use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use base64::Engine;
use resenha::{assets, ArticleAsset};
use std::io::Read;
use tower::ServiceExt;

#[test]
fn test_create_editorial_zip_structure() {
    let md = "# Artigo de Teste\n\n![Foto](attachments/imagem-1.png)\n";
    let fake_png_bytes = b"\x89PNG\r\n\x1a\nfakecontent";
    let b64 = base64::engine::general_purpose::STANDARD.encode(fake_png_bytes);

    let assets_list = vec![ArticleAsset {
        filename: "imagem-1.png".to_string(),
        original_url: "https://example.com/foto.png".to_string(),
        mime_type: "image/png".to_string(),
        data_base64: b64,
    }];

    let zip_bytes = assets::create_editorial_zip(md, &assets_list)
        .expect("Deveria gerar o pacote ZIP com sucesso");

    assert!(!zip_bytes.is_empty(), "ZIP não deve ser vazio");

    // Validação da estrutura interna usando zip::ZipArchive
    let reader = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(reader).expect("Deve ser um ZIP válido");

    // 1. main.md na raiz
    {
        let mut main_file = archive
            .by_name("main.md")
            .expect("ZIP deve conter main.md na raiz");
        let mut main_content = String::new();
        main_file
            .read_to_string(&mut main_content)
            .expect("Deve ler main.md");
        assert_eq!(main_content, md);
    }

    // 2. attachments/imagem-1.png
    {
        let mut img_file = archive
            .by_name("attachments/imagem-1.png")
            .expect("ZIP deve conter attachments/imagem-1.png");
        let mut img_content = Vec::new();
        img_file
            .read_to_end(&mut img_content)
            .expect("Deve ler imagem");
        assert_eq!(img_content, fake_png_bytes);
    }
}

#[tokio::test]
async fn test_cli_autodetection_and_quick_routes() {
    let app = resenha::server::create_app();

    // 1. Atalho /m/ deve retornar Markdown puro
    let req = Request::builder()
        .uri("/m/https://example.com")
        .header(header::USER_AGENT, "curl/8.5.0")
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_ne!(resp.status(), StatusCode::NOT_FOUND);

    // 2. Atalho /z/ deve rotear para o formato ZIP
    let req_zip = Request::builder()
        .uri("/z/https://example.com")
        .body(Body::empty())
        .unwrap();

    let resp_zip = app.clone().oneshot(req_zip).await.unwrap();
    assert_ne!(resp_zip.status(), StatusCode::NOT_FOUND);

    // 3. Atalho /j/ deve rotear para o formato JSON
    let req_json = Request::builder()
        .uri("/j/https://example.com")
        .body(Body::empty())
        .unwrap();

    let resp_json = app.clone().oneshot(req_json).await.unwrap();
    assert_ne!(resp_json.status(), StatusCode::NOT_FOUND);
}
