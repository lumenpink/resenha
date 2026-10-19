use crate::{ExtractedArticle, ResenhaError};

pub fn format_markdown(article: &ExtractedArticle) -> Result<String, ResenhaError> {
    let converter = htmd::HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style", "iframe", "noscript"])
        .build();

    let converted_body = converter.convert(&article.clean_html).map_err(|e| {
        ResenhaError::Formatting(format!("Erro ao converter HTML para Markdown: {e}"))
    })?;

    let mut frontmatter = String::from("---\n");
    frontmatter.push_str(&format!("title: {:?}\n", article.title));
    frontmatter.push_str(&format!("source: {:?}\n", article.source_url));
    frontmatter.push_str(&format!("domain: {:?}\n", article.domain));
    if let Some(ref author) = article.author {
        frontmatter.push_str(&format!("author: {:?}\n", author));
    }
    frontmatter.push_str(&format!("clipped_at: {:?}\n", article.clipped_at));
    frontmatter.push_str("generator: \"resenha\"\n");
    frontmatter.push_str("---\n\n");

    let original_badge = format!(
        "> ↗ **Fonte original:** [{}]({})\n\n",
        article.domain, article.source_url
    );

    let clean_body = converted_body.trim();
    let final_markdown = format!("{frontmatter}{original_badge}{clean_body}\n");

    Ok(final_markdown)
}
