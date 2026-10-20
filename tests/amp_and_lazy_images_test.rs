use resenha::{extractor, formatter};

#[test]
fn test_extractor_preserves_amp_images() {
    let html = r#"<!DOCTYPE html>
<html>
<head><title>Artigo com Imagem AMP</title></head>
<body>
<article>
<h1>Artigo com Imagem AMP</h1>
<p>Este é o primeiro parágrafo relevante da matéria jornalística para dar densidade textual suficiente ao teste.</p>
<figure>
    <amp-img src="https://s2-g1.glbimg.com/foto-teste.jpg" alt="Foto de Teste Nanini" width="1920" height="1080" layout="responsive"></amp-img>
    <figcaption>Foto de Teste Nanini — Foto: Divulgação</figcaption>
</figure>
<p>Segundo parágrafo detalhando o acontecimento para garantir que o algoritmo Readability mantenha o artigo completo.</p>
</article>
</body>
</html>"#;

    let target_url = "https://g1.globo.com/google/amp/teste.ghtml";
    let extracted = extractor::extract_and_sanitize(html, target_url)
        .expect("Extração deve ocorrer com sucesso");

    assert!(
        extracted.clean_html.contains("<img"),
        "HTML limpo deve conter tag <img> resultante de <amp-img>"
    );
    assert!(
        extracted.clean_html.contains("https://s2-g1.glbimg.com/foto-teste.jpg"),
        "HTML limpo deve preservar o link da imagem"
    );

    let md = formatter::format_markdown(&extracted).expect("Formatação deve ocorrer com sucesso");
    assert!(
        md.contains("![Foto de Teste Nanini](https://s2-g1.glbimg.com/foto-teste.jpg)")
            || md.contains("https://s2-g1.glbimg.com/foto-teste.jpg"),
        "Markdown deve conter a imagem extraída"
    );
}

#[test]
fn test_extractor_promotes_lazy_load_attributes() {
    let html = r#"<!DOCTYPE html>
<html>
<head><title>Artigo com Lazy Loading</title></head>
<body>
<article>
<h1>Artigo com Lazy Loading</h1>
<p>Este é o primeiro parágrafo relevante da matéria jornalística para dar densidade textual suficiente ao teste.</p>
<img src="data:image/svg+xml;base64,PHN2Zy..." data-src="https://exemplo.com/imagem-real.jpg" alt="Imagem Preguiçosa" />
<p>Segundo parágrafo detalhando o acontecimento para garantir que o algoritmo Readability mantenha o artigo completo.</p>
</article>
</body>
</html>"#;

    let target_url = "https://exemplo.com/artigo";
    let extracted = extractor::extract_and_sanitize(html, target_url)
        .expect("Extração deve ocorrer com sucesso");

    assert!(
        extracted.clean_html.contains("https://exemplo.com/imagem-real.jpg"),
        "Atributo data-src deve ser promovido para src"
    );
    assert!(
        !extracted.clean_html.contains("data:image/svg+xml"),
        "Placeholder data:image não deve permanecer no src final"
    );
}
