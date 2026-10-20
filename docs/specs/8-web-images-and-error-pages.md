# Spec: Páginas de Erro Tipográficas e Imagens Embutidas na Leitura Web

- **Ticket**: #8
- **Tipo**: Fix / Usabilidade e Soberania
- **Data**: 2026-10-20

## 1. Problema e Motivação
1. **Quebra de imagens na visualização web:** O pipeline de download de assets reescreve links de imagens para caminhos relativos ao arquivo ZIP (`attachments/imagem-X.jpg`). Quando a leitora acessa a página HTML de leitura no navegador, este tenta requisitar `https://resenha.lohn.in/attachments/imagem-1.jpg`. O catch-all do servidor interpreta `attachments/...` como uma URL externa e tenta buscar `https://attachments/...`, gerando falha de DNS e erro 502 no browser.
2. **Erros sem formatação ("tela crua"):** Quando uma requisição falha (como 404 da fonte original ou erro de rede), o servidor devolve uma string de texto puro (`Erro no processamento: ...`) com status 502/404, sem estilização, sem contexto amigável e sem a identidade visual do Resenha.
3. **Imagens de destaque ignoradas em portais como O Globo:** Fotos principais situadas em cabeçalhos (como `<section class="content--header"> <div class="content-top-image"> <figure>...`) ficam fora da tag `<article>`. O extrator Readability foca em `<article>` e descarta essas imagens de abertura.

## 2. Requisitos e Contratos

### 2.1. Imagens Embutidas Soberanas (Inlined Base64)
- Na função `formatter::render_reading_page`:
  - As referências a `attachments/{filename}` geradas a partir do Markdown devem ser convertidas em data URIs base64 (`data:{mime_type};base64,{data_base64}`) correspondentes aos assets contidos em `article.assets`.
  - Caso o asset não esteja disponível em base64, deve preservar a URL remota original de fallback.
  - O Markdown bruto (.md) e o pacote ZIP (.zip) permanecem usando caminhos relativos `attachments/{filename}` para portabilidade em vaults como Obsidian.

### 2.2. Página de Erro Tipográfica (`render_error_page`)
- Implementar `formatter::render_error_page(status: StatusCode, title: &str, message: &str, details: Option<&str>) -> String`.
- Seguir a identidade visual do Resenha:
  - Fontes clássicas serifadas (`Charter`, `Georgia`), suporte a Dark e Light mode automático.
  - Logotipo `✂ resenha` e subtítulo.
  - Badge numérico do código HTTP (ex.: 404, 502, 400).
  - Mensagem explicativa em português claro com link de retorno à página inicial.
  - Bloco monospace recolhível ou formatado para detalhes técnicos da mensagem de erro.
- O handler de erro (`AppError::into_response`) deve verificar o cabeçalho `Accept`:
  - Se contiver `text/html`, responde com HTML da `render_error_page`.
  - Se for cliente CLI (`curl`, `wget`) ou formato JSON/MD requisitado, responde em formato textual conciso ou JSON.

### 2.3. Roteamento e Proteção do Catch-all
- Requisições para caminhos inválidos ou pseudo-URLs (como `/attachments/...` ou rotas inexistentes) devem ser barradas e retornar HTTP 404 diretamente, sem disparar requisições DNS/HTTP externas.
- URLs válidas no catch-all devem conter pelo menos um ponto no host (ex.: `dominio.tld`) ou formato `http(s)://`.

### 2.4. Promoção de Imagens de Destaque no DOM (`preprocess_html`)
- Detectar figuras de destaque presentes fora de `<article>` (ex.: `div.content-top-image figure`, `figure.content-featured-figure`, `header figure`).
- Injetar tais figuras no início do elemento `<article>`, permitindo que o Readability as considere parte do conteúdo do artigo.
