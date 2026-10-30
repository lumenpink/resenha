# Uso e Configuração — resenha

## 1. Linha de Comando (CLI)

```bash
# Extrair direto no terminal
resenha "https://exemplo.com/noticia"

# Salvar como Markdown
resenha "https://exemplo.com/noticia" -o artigo.md

# Saída em JSON
resenha "https://exemplo.com/noticia" --format json
```

## 2. Modo Servidor (Container ou Local)

```bash
# Iniciar localmente
resenha serve --port 3030

# Iniciar via Docker Compose
docker compose up -d

# Compilação otimizada com LTO e strip
cargo build --release

# Compilação estática universal (static-pie) contra musl libc
cargo build --release --target x86_64-unknown-linux-musl
```

## 3. Variáveis de Ambiente

* `RESENHA_ENABLE_FALLBACK`: Ativa o fallback de extração quando a origem responder 403/401 (`true` por padrão).
* `RESENHA_FALLBACK_URL`: Template do resolvedor de fallback (padrão: `https://r.jina.ai/{url}`).
* `RESENHA_TIMEOUT_SECS`: Tempo limite de requisições em segundos (padrão: 90s).
* `DOWNLOAD_ASSETS`: Baixa e embute imagens locais em base64 (`true` por padrão).

## 4. Formato do Markdown Gerado

```markdown
---
title: "Título do Artigo"
source: "https://exemplo.com/noticia"
domain: "exemplo.com"
author: "Nome do Autor"
clipped_at: 2026-10-19T14:45:00Z
generator: "resenha"
---

> ↗ **Fonte original:** [exemplo.com](https://exemplo.com/noticia)

# Título do Artigo

Texto limpo, sem anúncios, sem scripts e com parágrafos preservados...
```
