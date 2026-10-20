# Especificação Técnica: resenha
**Subsistema de Extração, Sanitização e Destilação Tipográfica na Borda**

- **Ticket de Referência**: #1
- **Ecossistema**: 2LP // A Banca (`fuxico.lumen.pink` / `resenha`)
- **Categoria**: 🌐 IndieWeb / ✂ Editorial
- **Linguagem**: Rust (Edição 2021)
- **Licença**: GPL-3.0
- **Governança**: Norma GOST v1.1.0 (Perfil Lumen)

---

## 1. Manifesto & Propósito

A web comercial converteu o ato de ler em uma trincheira contra popups invasivos, banners de newsletter, vídeos em autoplay, rastreadores comportamentais e excesso de scripts. Ferramentas legadas como o Readability foram descontinuadas, e alternativas self-hosted tradicionais frequentemente exigem bancos relacionais pesados, dependências inchadas e navegadores headless que consomem gigabytes de memória.

O **`resenha`** é um microserviço e ferramenta de linha de comando (CLI) em Rust puro, estritamente containerizado, com consumo residente inferior a 15 MB de RAM. Seu propósito é extrair o conteúdo essencial de qualquer artigo ou notícia da web aberta e devolvê-lo na forma de **Markdown estrito**, legível e soberano, com metadados estruturados e preservação explícita do **link da fonte original**.

O `resenha` foi projetado para operar com dupla vocação:
1. **Independente (Standalone)**: Como utilitário CLI ou serviço HTTP que entrega Markdown ou JSON estruturado.
2. **Integrado ao Fuxico (`fuxico.lumen.pink`)**: Alimentando o `MarkdownRenderer` e o visualizador nativo do Fuxico, gerando arquivos estáticos imutáveis e permitindo exibição destacada do link original.

---

## 2. Decisões Arquiteturais & Limites de Escopo

### 2.1. O que faz parte do escopo:
- **Extração Heurística de Conteúdo**: Avaliação da densidade textual do DOM usando a heurística Mozilla Readability em Rust puro para identificar título, autor e corpo principal.
- **Normalização Prévia de Mídia e DOM**: Pré-processamento antes da extração para normalizar tags `<amp-img>` em tags `<img>` padrão e promover atributos de lazy-loading (`data-src`, `data-original`, `data-lazy-src`, `srcset`) para `src`, garantindo que imagens de páginas AMP e sites modernos não sejam expurgadas.
- **Sanitização Estrita (Anti-XSS e Anti-Tracking)**: Remoção sistemática de `<script>`, `<style>`, `<iframe>`, `<object>`, `<embed>`, `<form>`, manipuladores de eventos JavaScript (`onload`, `onclick`), além de limpeza de parâmetros de espionagem comercial em URLs (`utm_*`, `fbclid`, `gclid`).
- **Conversão para CommonMark Estrito**: Transformação do HTML limpo para Markdown com blocos de citação, cabeçalhos, código e listas preservados.
- **Preservação Canônica da Fonte**: Injeção obrigatória do link da fonte original tanto no Frontmatter YAML quanto em bloco visual de destaque no início do Markdown.
- **Microserviço HTTP com Axum**: Endpoints de leitura pass-through e verificação de saúde (`/healthz`).
- **Containerização Hermética**: Dockerfile multi-stage com usuário não-root e healthcheck nativo.

### 2.2. O que NÃO faz parte do escopo (e justificativas):
- **Sem Navegadores Headless (Chromium / Playwright)**: Preserva binário enxuto, baixo consumo de memória e inicialização em milissegundos.
- **Sem Banco de Dados Relacional**: Opera em memória (pass-through) ou persistência direta em flat-files.
- **Sem Evasão de Paywalls Fechados**: Focado em higiene visual, legibilidade e preservação editorial em páginas públicas.

---

## 3. Contratos de Interface (I/O)

### 3.1. Interface de Linha de Comando (CLI)

```bash
# Extrair e imprimir Markdown no stdout
resenha "https://exemplo.com/noticia"

# Salvar em arquivo específico
resenha "https://exemplo.com/noticia" -o artigo.md

# Saída em JSON estruturado
resenha "https://exemplo.com/noticia" --format json

# Iniciar servidor HTTP daemon
resenha serve --port 3030 --bind 0.0.0.0
```

### 3.2. Interface HTTP (Microserviço)

#### `GET /healthz`
- **Finalidade**: Verificação de integridade para Docker/Kube/Caddy.
- **Resposta**: `200 OK`, `text/plain: ok`.

#### `GET /` (Sem parâmetros)
- **Finalidade**: Interface web editorial minimalista com formulário para submissão de URL, seleção de formato (`html` para leitura na tela ou `md` para download) e atalho para bookmarklet.
- **Resposta**: `200 OK`, `text/html; charset=utf-8`.

#### `GET /` ou `GET /resenha?url={ALVO}`
- **Parâmetros**:
  - `url` (obrigatório se invocando a extração via query string): URL codificada do artigo original.
  - `format` (opcional): `html` (leitura tipográfica na tela), `md` (CommonMark com Frontmatter), `json` ou `zip` (pacote editorial completo).

#### Acessos Rápidos por Rota (Path Routing)
O servidor suporta atalhos diretos sem a necessidade de query strings:
- `GET /https://site.com/...` ou `GET /site.com/...`: Formato padrão (HTML no navegador, Markdown no terminal).
- `GET /m/https://site.com/...` ou `GET /md/...`: Markdown puro direto (`text/markdown`).
- `GET /z/https://site.com/...` ou `GET /zip/...`: Pacote editorial `.zip` (`application/zip`).
- `GET /j/https://site.com/...` ou `GET /json/...`: JSON estruturado (`application/json`).

#### Autodetecção de Terminal (CLI)
- Clientes de linha de comando (`curl`, `wget`, `httpie`) ou requisições sem `text/html` no cabeçalho `Accept` recebem automaticamente **Markdown puro** no stdout, sem necessidade de parâmetros.
- Navegadores com `Accept: text/html` recebem a interface web de leitura tipográfica.

#### Pacote Editorial Canônico (.zip)
Ao solicitar o formato `zip` (via rota `/z/...`, query `?format=zip` ou botão web):
- O pacote contém na raiz o arquivo `main.md` com YAML Frontmatter e links de mídia apontando para `attachments/<arquivo>`.
- O pacote contém a pasta `attachments/` com todas as imagens e ativos baixados em resolução original.
- Totalmente compatível com Joaju e Obsidian out-of-the-box.
- **Códigos de Resposta**:
  - `200 OK`: Extração concluída ou página inicial renderizada.
  - `400 Bad Request`: Parâmetro `url` inválido ou vazio.
  - `502 Bad Gateway`: Falha de conexão ou timeout na página de origem.
  - `422 Unprocessable Entity`: Conteúdo não pôde ser extraído como artigo.

---

## 4. Estrutura do Markdown Gerado

O Markdown gerado pelo `resenha` deve obrigatoriamente seguir a seguinte estrutura canônica:

```markdown
---
title: "Título do Artigo Extraído"
source: "https://exemplo.com/artigo-original"
domain: "exemplo.com"
author: "Nome do Autor (se disponível)"
clipped_at: 2026-10-19T14:45:00Z
generator: "resenha"
---

> ↗ **Fonte original:** [exemplo.com](https://exemplo.com/artigo-original)

# Título do Artigo Extraído

Texto destilado, sem scripts, sem anúncios e com formatação preservada...
```

---

## 5. Arquitetura de Módulos (Rust)

```text
resenha/
├── docs/specs/resenha.md        # Esta especificação
├── tests/                       # Testes de integração e fixtures
│   ├── fixtures/
│   │   ├── noisy_article.html   # HTML com scripts, popups e trackers
│   │   └── clean_expected.md    # Markdown esperado após destilação
│   └── pipeline_test.rs         # Testes de contrato do motor
└── src/
    ├── lib.rs                   # Biblioteca reutilizável do motor
    ├── main.rs                  # CLI e despacho de subcomandos
    ├── fetcher.rs               # Cliente HTTP (reqwest + rustls)
    ├── extractor.rs             # Mozilla Readability + Ammonia
    ├── formatter.rs             # Conversão HTML -> Markdown + Frontmatter
    └── server.rs                # Servidor HTTP com Axum
```

---

## 6. Containerização & Infraestrutura (Padrão GOST)

- **Dockerfile**:
  - Estágio de compilação: `rust:1-slim-bookworm` (com cache de dependências).
  - Estágio de runtime: `debian:bookworm-slim` com `ca-certificates`.
  - Usuário de execução: `USER 1000:1000`.
  - Exposição de porta: `EXPOSE 3030`.
- **docker-compose.yml**:
  - Mapeamento de porta `3030:3030`.
  - Healthcheck ativo contra `http://localhost:3030/healthz`.
