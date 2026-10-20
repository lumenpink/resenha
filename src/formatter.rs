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

pub fn render_home_page() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>✂ resenha // destilação tipográfica na borda</title>
    <style>
        :root {
            --bg: #fdfbf7;
            --text: #2c2825;
            --muted: #736b63;
            --border: #e0d8ce;
            --accent: #b83232;
            --card-bg: #ffffff;
            --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            --font-serif: "Charter", "Georgia", "Iowan Old Style", serif;
            --font-mono: ui-monospace, "Cascadia Code", "Fira Code", monospace;
        }
        @media (prefers-color-scheme: dark) {
            :root {
                --bg: #141414;
                --text: #dedede;
                --muted: #949494;
                --border: #2a2a2a;
                --accent: #ff6b6b;
                --card-bg: #1e1e1e;
            }
        }
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            background-color: var(--bg);
            color: var(--text);
            font-family: var(--font-sans);
            line-height: 1.6;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            padding: 1.5rem;
        }
        .container {
            width: 100%;
            max-width: 580px;
        }
        .header {
            margin-bottom: 2rem;
            text-align: center;
        }
        .logo {
            font-family: var(--font-serif);
            font-size: 2.5rem;
            font-weight: 700;
            letter-spacing: -0.03em;
            color: var(--text);
            margin-bottom: 0.25rem;
        }
        .tagline {
            color: var(--muted);
            font-size: 0.95rem;
        }
        .card {
            background: var(--card-bg);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 1.75rem;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.04);
        }
        label {
            display: block;
            font-size: 0.85rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--muted);
            margin-bottom: 0.5rem;
        }
        .input-group {
            margin-bottom: 1.25rem;
        }
        input[type="url"] {
            width: 100%;
            padding: 0.8rem 1rem;
            font-size: 1rem;
            border: 1px solid var(--border);
            border-radius: 6px;
            background: var(--bg);
            color: var(--text);
            outline: none;
            transition: border-color 0.15s;
        }
        input[type="url"]:focus {
            border-color: var(--accent);
        }
        .formats-group {
            display: flex;
            gap: 1.5rem;
            margin-bottom: 1.5rem;
        }
        .radio-option {
            display: flex;
            align-items: center;
            gap: 0.4rem;
            font-size: 0.95rem;
            cursor: pointer;
        }
        button[type="submit"] {
            width: 100%;
            background: var(--text);
            color: var(--bg);
            border: none;
            padding: 0.9rem;
            font-size: 1rem;
            font-weight: 600;
            border-radius: 6px;
            cursor: pointer;
            transition: opacity 0.15s;
        }
        button[type="submit"]:hover {
            opacity: 0.9;
        }
        .footer-note {
            margin-top: 1.5rem;
            text-align: center;
            font-size: 0.85rem;
            color: var(--muted);
            font-family: var(--font-mono);
        }
        .footer-note a {
            color: var(--accent);
            text-decoration: none;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="logo">✂ resenha</div>
            <div class="tagline">extração, sanitização e destilação tipográfica na borda</div>
        </div>
        <div class="card">
            <form action="/" method="GET">
                <div class="input-group">
                    <label for="url">Endereço do Artigo</label>
                    <input type="url" id="url" name="url" placeholder="https://exemplo.com/artigo-poluido" required autofocus>
                </div>
                <div class="input-group">
                    <label>Modo de Destilação</label>
                    <div class="formats-group">
                        <label class="radio-option">
                            <input type="radio" name="format" value="html" checked>
                            Leitura Tipográfica (HTML)
                        </label>
                        <label class="radio-option">
                            <input type="radio" name="format" value="md">
                            Markdown Puro (.md)
                        </label>
                        <label class="radio-option">
                            <input type="radio" name="format" value="zip">
                            Pacote Editorial (.zip)
                        </label>
                    </div>
                </div>
                <button type="submit">✂ Resenhar Artigo</button>
            </form>
        </div>
        <div class="footer-note">
            <span>2LP // A Banca &bull; soberania da leitura</span>
        </div>
    </div>
</body>
</html>"#
}

pub fn markdown_to_html(md: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(md, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

pub fn strip_frontmatter(md: &str) -> &str {
    if let Some(stripped) = md.strip_prefix("---\n") {
        if let Some(end_idx) = stripped.find("\n---\n") {
            return &stripped[end_idx + 5..];
        }
    }
    md
}

pub fn render_error_page(
    status: axum::http::StatusCode,
    title: &str,
    message: &str,
    details: Option<&str>,
) -> String {
    let title_escaped = html_escape(title);
    let message_escaped = html_escape(message);
    let status_code = status.as_u16();
    let details_section = if let Some(d) = details.filter(|s| !s.is_empty()) {
        let d_escaped = html_escape(d);
        format!(
            r#"<div class="error-details-box">
                <div class="error-details-label">Detalhes Técnicos:</div>
                <pre class="error-details"><code>{d_escaped}</code></pre>
            </div>"#
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{status_code} // {title_escaped} - resenha</title>
    <style>
        :root {{
            --bg: #fdfbf7;
            --text: #2c2825;
            --muted: #736b63;
            --border: #e0d8ce;
            --accent: #b83232;
            --card-bg: #ffffff;
            --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            --font-serif: "Charter", "Georgia", "Iowan Old Style", serif;
            --font-mono: ui-monospace, "Cascadia Code", "Fira Code", monospace;
        }}
        @media (prefers-color-scheme: dark) {{
            :root {{
                --bg: #141414;
                --text: #dedede;
                --muted: #949494;
                --border: #2a2a2a;
                --accent: #ff6b6b;
                --card-bg: #1e1e1e;
            }}
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background-color: var(--bg);
            color: var(--text);
            font-family: var(--font-sans);
            line-height: 1.6;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            padding: 1.5rem;
        }}
        .container {{
            width: 100%;
            max-width: 580px;
        }}
        .header {{
            margin-bottom: 2rem;
            text-align: center;
        }}
        .logo {{
            font-family: var(--font-serif);
            font-size: 2.2rem;
            font-weight: 700;
            letter-spacing: -0.03em;
            color: var(--text);
            margin-bottom: 0.25rem;
            text-decoration: none;
            display: inline-block;
        }}
        .tagline {{
            color: var(--muted);
            font-size: 0.9rem;
        }}
        .card {{
            background: var(--card-bg);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 2rem;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.04);
            text-align: center;
        }}
        .status-badge {{
            display: inline-block;
            background: var(--accent);
            color: #ffffff;
            font-family: var(--font-mono);
            font-size: 0.85rem;
            font-weight: 700;
            padding: 0.25rem 0.65rem;
            border-radius: 4px;
            margin-bottom: 1.25rem;
        }}
        .error-title {{
            font-family: var(--font-serif);
            font-size: 1.6rem;
            font-weight: 700;
            color: var(--text);
            margin-bottom: 0.75rem;
            line-height: 1.3;
        }}
        .error-message {{
            color: var(--muted);
            font-size: 1rem;
            margin-bottom: 1.5rem;
            line-height: 1.6;
        }}
        .error-details-box {{
            text-align: left;
            margin-bottom: 1.5rem;
            background: var(--bg);
            border: 1px solid var(--border);
            border-radius: 6px;
            padding: 0.85rem 1rem;
        }}
        .error-details-label {{
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--muted);
            margin-bottom: 0.35rem;
        }}
        .error-details {{
            font-family: var(--font-mono);
            font-size: 0.8rem;
            color: var(--text);
            white-space: pre-wrap;
            word-break: break-all;
            overflow-x: auto;
        }}
        .actions {{
            display: flex;
            gap: 1rem;
            justify-content: center;
        }}
        .btn-home {{
            display: inline-block;
            background: var(--text);
            color: var(--bg);
            text-decoration: none;
            padding: 0.8rem 1.4rem;
            font-size: 0.95rem;
            font-weight: 600;
            border-radius: 6px;
            transition: opacity 0.15s;
        }}
        .btn-home:hover {{
            opacity: 0.9;
        }}
        .footer-note {{
            margin-top: 1.5rem;
            text-align: center;
            font-size: 0.85rem;
            color: var(--muted);
            font-family: var(--font-mono);
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <a href="/" class="logo">✂ resenha</a>
            <div class="tagline">destilação tipográfica soberana na borda</div>
        </div>
        <div class="card">
            <div class="status-badge">{status_code}</div>
            <h1 class="error-title">{title_escaped}</h1>
            <p class="error-message">{message_escaped}</p>
            {details_section}
            <div class="actions">
                <a href="/" class="btn-home">← Voltar à página inicial</a>
            </div>
        </div>
        <div class="footer-note">
            <span>2LP // A Banca &bull; soberania da leitura</span>
        </div>
    </div>
</body>
</html>"#
    )
}

pub fn render_reading_page(
    article: &ExtractedArticle,
    md_content: &str,
    fuxico_api_url: Option<&str>,
) -> String {
    let title_escaped = html_escape(&article.title);
    let domain_escaped = html_escape(&article.domain);
    let source_escaped = html_escape(&article.source_url);
    let author_escaped = article
        .author
        .as_deref()
        .map(html_escape)
        .unwrap_or_default();
    let encoded_url = urlencoding_encode(&article.source_url);

    let author_markup = if !author_escaped.is_empty() {
        format!(
            "<span>Por <strong>{}</strong></span> &bull; ",
            author_escaped
        )
    } else {
        String::new()
    };

    let fuxico_button = if fuxico_api_url.is_some() {
        r#"<button id="btn-fuxicar" class="btn-action" onclick="fuxicarArtigo()" title="Publicar como Fuxico Doc no cofre">⚡ Fuxicar</button>"#
    } else {
        ""
    };

    let body_md = strip_frontmatter(md_content).trim();
    let mut rendered_body = markdown_to_html(body_md);

    // Na visualização web, substituir referências relativas "attachments/..." por dados inline base64
    // Isso torna a página soberana, autônoma e evita que o navegador tente requisitar /attachments/...
    for asset in &article.assets {
        let needle = format!("attachments/{}", asset.filename);
        let replacement = if !asset.data_base64.is_empty() {
            format!("data:{};base64,{}", asset.mime_type, asset.data_base64)
        } else {
            asset.original_url.clone()
        };
        rendered_body = rendered_body.replace(&needle, &replacement);
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title_escaped} // resenha</title>
    <style>
        :root {{
            --bg: #fefdfb;
            --text: #22201d;
            --muted: #736b63;
            --border: #e8e2d8;
            --accent: #b83232;
            --link: #1c5ba6;
            --quote-border: #d4cbbe;
            --font-serif: "Charter", "Georgia", "Iowan Old Style", serif;
            --font-mono: ui-monospace, "Cascadia Code", "Fira Code", monospace;
            --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        }}
        @media (prefers-color-scheme: dark) {{
            :root {{
                --bg: #141414;
                --text: #e4e2de;
                --muted: #949089;
                --border: #2a2825;
                --accent: #ff6b6b;
                --link: #74a8e8;
                --quote-border: #3d3a36;
            }}
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background-color: var(--bg);
            color: var(--text);
            font-family: var(--font-serif);
            font-size: 1.18rem;
            line-height: 1.75;
            padding: 1.5rem 1rem 6rem;
            text-rendering: optimizeLegibility;
        }}
        .top-bar {{
            position: sticky;
            top: 0;
            background: var(--bg);
            border-bottom: 1px solid var(--border);
            padding: 0.75rem 0;
            margin-bottom: 3rem;
            font-family: var(--font-sans);
            font-size: 0.85rem;
            z-index: 100;
        }}
        .top-bar-inner {{
            max-width: 68ch;
            margin: 0 auto;
            display: flex;
            justify-content: space-between;
            align-items: center;
            gap: 1rem;
        }}
        .brand {{
            font-weight: 700;
            color: var(--text);
            text-decoration: none;
        }}
        .actions-bar {{
            display: flex;
            gap: 0.75rem;
            align-items: center;
        }}
        .btn-action {{
            color: var(--muted);
            text-decoration: none;
            padding: 0.35rem 0.65rem;
            border-radius: 4px;
            border: 1px solid var(--border);
            background: transparent;
            font-size: 0.85rem;
            cursor: pointer;
            transition: all 0.15s;
        }}
        .btn-action:hover {{
            color: var(--text);
            border-color: var(--text);
        }}
        #btn-fuxicar {{
            background: var(--text);
            color: var(--bg);
            border-color: var(--text);
            font-weight: 600;
        }}
        #btn-fuxicar:hover {{
            opacity: 0.85;
        }}
        article {{
            max-width: 68ch;
            margin: 0 auto;
        }}
        header.article-header {{
            margin-bottom: 2.5rem;
            padding-bottom: 1.5rem;
            border-bottom: 1px solid var(--border);
        }}
        h1.article-title {{
            font-size: 2.3rem;
            line-height: 1.25;
            letter-spacing: -0.02em;
            margin-bottom: 1rem;
            color: var(--text);
        }}
        .article-meta {{
            font-family: var(--font-sans);
            font-size: 0.9rem;
            color: var(--muted);
        }}
        .content p {{
            margin-bottom: 1.5rem;
        }}
        .content h2, .content h3, .content h4 {{
            margin: 2.5rem 0 1rem;
            line-height: 1.3;
            letter-spacing: -0.01em;
        }}
        .content blockquote {{
            margin: 2rem 0;
            padding-left: 1.25rem;
            border-left: 3px solid var(--quote-border);
            color: var(--muted);
            font-style: italic;
        }}
        .content ul, .content ol {{
            margin: 1.5rem 0 1.5rem 1.5rem;
        }}
        .content li {{
            margin-bottom: 0.5rem;
        }}
        .content a {{
            color: var(--link);
            text-decoration-thickness: 1px;
            text-underline-offset: 3px;
        }}
        .content img {{
            max-width: 100%;
            height: auto;
            border-radius: 4px;
            margin: 2rem 0;
            display: block;
        }}
        .content pre {{
            background: var(--border);
            padding: 1rem;
            border-radius: 6px;
            overflow-x: auto;
            font-family: var(--font-mono);
            font-size: 0.9rem;
            margin: 1.5rem 0;
        }}
        .content code {{
            font-family: var(--font-mono);
            font-size: 0.9em;
            background: var(--border);
            padding: 0.15rem 0.35rem;
            border-radius: 3px;
        }}
    </style>
</head>
<body>
    <div class="top-bar">
        <div class="top-bar-inner">
            <a href="/" class="brand">✂ resenha</a>
            <div class="actions-bar">
                {fuxico_button}
                <a href="{source_escaped}" target="_blank" rel="noopener noreferrer" class="btn-action" title="Abrir artigo original na fonte">↗ Original</a>
                <a href="/?url={encoded_url}&format=md" class="btn-action" title="Baixar versão Markdown pura">⬇ Markdown</a>
                <a href="/?url={encoded_url}&format=zip" class="btn-action" title="Baixar pacote editorial ZIP (main.md + anexos)">📦 Pacote ZIP</a>
                <a href="/" class="btn-action" title="Destilar outro artigo">↩ Nova Resenha</a>
            </div>
        </div>
    </div>
    <article>
        <header class="article-header">
            <h1 class="article-title">{title_escaped}</h1>
            <div class="article-meta">
                {author_markup}
                <span>Fonte: <strong>{domain_escaped}</strong></span>
            </div>
        </header>
        <div class="content">
            {rendered_body}
        </div>
    </article>

    <script>
    async function fuxicarArtigo() {{
        const btn = document.getElementById('btn-fuxicar');
        if (!btn) return;
        const orig = btn.innerText;
        btn.disabled = true;
        btn.innerText = '⚡ Enviando...';
        try {{
            const target = new URL(window.location.href).searchParams.get('url');
            if (!target) throw new Error('URL ausente');
            const res = await fetch('/api/fuxicar?url=' + encodeURIComponent(target), {{ method: 'POST' }});
            const data = await res.json();
            if (data.short_url) {{
                await navigator.clipboard.writeText(data.short_url);
                alert('Fofoca publicada no Fuxico! Link copiado para a área de transferência:\n' + data.short_url);
                btn.innerText = '✓ Fuxicado!';
            }} else {{
                alert('Erro ao fuxicar: ' + (data.error || 'Falha desconhecida'));
                btn.innerText = orig;
            }}
        }} catch (err) {{
            alert('Falha na comunicação: ' + err.message);
            btn.innerText = orig;
        }} finally {{
            setTimeout(() => {{ btn.disabled = false; btn.innerText = orig; }}, 4000);
        }}
    }}
    </script>
</body>
</html>"#
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn urlencoding_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
