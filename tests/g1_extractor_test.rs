use resenha::extractor;

const G1_FIXTURE: &str = include_str!("fixtures/g1_sample.html");
const TARGET_URL: &str = "https://g1.globo.com/politica/eleicoes/2026/noticia/2026/10/21/campanha-diz-que-lula-nao-se-opoe-a-comdocoes-de-flavio-bolsonaro-para-quebra-de-sigilo.ghtml";

#[test]
fn test_g1_full_article_extraction_not_truncated() {
    let article = extractor::extract_and_sanitize(G1_FIXTURE, TARGET_URL)
        .expect("O extrator deve processar o artigo do G1 com sucesso");

    println!("TÍTULO: {}", article.title);
    println!("CONTEÚDO EXTRAÍDO:\n{}", article.clean_html);

    let p_count = article.clean_html.matches("<p").count();
    println!("Total de parágrafos extraídos: {}", p_count);

    // O artigo original na fixture possui 10 parágrafos.
    // Antes da correção, o resenha extraía apenas 1 parágrafo truncado.
    assert!(
        p_count >= 8,
        "O artigo do G1 foi truncado! Esperado >= 8 parágrafos, obtido {}",
        p_count
    );

    // Garantir que trechos do início, meio e fim estão presentes no corpo limpo
    assert!(
        article.clean_html.contains("não colocará obstáculos"),
        "O primeiro parágrafo deve estar presente no conteúdo extraído"
    );
    assert!(
        article.clean_html.contains("trocas de declarações"),
        "Parágrafos intermediários devem estar presentes no conteúdo extraído"
    );
    assert!(
        article
            .clean_html
            .contains("pronta para averiguação judicial"),
        "O último parágrafo deve estar presente no conteúdo extraído"
    );
}
