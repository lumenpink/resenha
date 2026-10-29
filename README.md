# ✂ resenha

**Subsistema de Extração, Sanitização e Destilação Tipográfica na Borda**

* **Ecossistema:** 2LP // A Banca (`fuxico.lumen.pink` / `resenha`)
* **Categoria:** 🌐 IndieWeb / ✂ Editorial
* **Linguagem:** Rust (Edição 2021)
* **Licença:** [GPL-3.0](./LICENSE)
* **Governança:** Norma [GOST](https://codeberg.org/lumenpink/gost) v1.1.0 (Perfil Lumen)

---

## 1. Manifesto

A web comercial transformou o ato de ler em uma trincheira contra popups, popunders, vídeos em autoplay, rastreadores e telas de consentimento abusivas. Alternativas clássicas como o Readability foram extintas, e ferramentas self-hosted comuns exigem infraestrutura pesada de banco de dados e navegadores headless inchados.

O **`resenha`** é o braço editorial da Banca: um microserviço e utilitário de linha de comando em Rust puro, ultrarrápido, com consumo residente inferior a 15 MB de RAM. Ele extrai o miolo textual de qualquer página web aberta e entrega em **Markdown estrito** com YAML Frontmatter e preservação obrigatória do **link da fonte original**.

Opera tanto como ferramenta de terminal independente quanto como motor integrado ao **Fuxico** (`fuxico.lumen.pink`).

---

## 2. Instalação & Uso

### Linha de Comando (CLI)

```bash
# Extrair direto no terminal
resenha "https://exemplo.com/noticia"

# Salvar como Markdown
resenha "https://exemplo.com/noticia" -o artigo.md

# Saída em JSON
resenha "https://exemplo.com/noticia" --format json
```

### Modo Servidor (Container ou Local)

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

### Variáveis de Ambiente

* `RESENHA_ENABLE_FALLBACK`: Ativa o fallback de extração quando a origem responder 403/401 (`true` por padrão).
* `RESENHA_FALLBACK_URL`: Template do resolvedor de fallback (padrão: `https://r.jina.ai/{url}`).
* `RESENHA_TIMEOUT_SECS`: Tempo limite de requisições em segundos (padrão: 15s).
* `DOWNLOAD_ASSETS`: Baixa e embute imagens locais em base64 (`true` por padrão).

---

## 3. Formato do Markdown Gerado

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

---

## 4. Governança & Conformidade GOST

Este repositório é gerenciado estritamente pelas diretrizes **GOST DD-TDD** e pela ferramenta **CCCP**:

```bash
# Auditar conformidade técnica
cccp check --strict
```

## 📄 Licença

Distribuído sob a licença [GPL-3.0](./LICENSE).
