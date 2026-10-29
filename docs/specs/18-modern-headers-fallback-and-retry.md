# Spec: Cabeçalhos Modernos, Fallback de Extração e Ação de Retry com Erros Amigáveis

- **Ticket**: #18
- **Tipo**: Feat / Usabilidade e Resiliência
- **Data**: 2026-10-29

## 1. Problema e Motivação
1. **Bloqueio por WAFs em Portais de Notícias (ex.: Akamai / UOL):** Requisições diretas com User-Agent fixo de robô sofrem bloqueio sumário (HTTP 403 Forbidden). Em portais com proteção de borda restritiva, requisições diretas de certos ASNs/IPs são barradas mesmo com cabeçalhos simulados.
2. **Página de Erro com Ação Limitada:** Ao encontrar falha, a leitora dispõe unicamente do link para voltar ao início, o que é contraproducente para erros intermitentes (como timeouts de conexão).
3. **Ausência de Diferenciação Visual e Didática de Erros:** Erros temporários (timeouts, quedas pontuais de rede) e bloqueios definitivos (paywall rígido 403, páginas removidas 404) não deixam evidente se cabe nova tentativa imediata.

## 2. Requisitos e Contratos

### 2.1. Cabeçalhos de Navegador Moderno no `fetcher`
- O cliente HTTP principal (`reqwest::Client`) deve enviar cabeçalhos padrão de navegador de última geração:
  - `User-Agent`: Navegador moderno em plataforma desktop estável.
  - `Accept`: `text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8`
  - `Accept-Language`: `pt-BR,pt;q=0.9,en-US;q=0.8,en;q=0.7`
  - `Upgrade-Insecure-Requests`: `1`
  - `Sec-Fetch-Dest`: `document`
  - `Sec-Fetch-Mode`: `navigate`
  - `Sec-Fetch-Site`: `none`

### 2.2. Fallback de Extração Resiliente
- Quando a requisição direta retornar erro de bloqueio (HTTP 403 ou 401):
  - Se habilitado via variável de ambiente (por padrão habilitado, podendo ser desativado com `RESENHA_ENABLE_FALLBACK=0` ou `false`), ou apontado para URL customizada com `RESENHA_FALLBACK_URL`:
    - URL padrão de fallback: `https://r.jina.ai/{url}` (com header `X-Return-Format: html`).
    - Dispara a requisição ao resolvedor de fallback com timeout configurado.
    - O conteúdo HTML recuperado é encaminhado normalmente ao extrator e formatador do Resenha.
  - Se o fallback também falhar ou estiver desabilitado, o erro de bloqueio é emitido normalmente.

### 2.3. Ação de Retry na Página de Erro
- A função `render_error_page` deve conter o botão "Tentar novamente":
  ```html
  <button type="button" class="btn-retry" onclick="window.location.reload()">↻ Tentar novamente</button>
  ```
- O botão deve acompanhar o botão já existente de retorno ao início.

### 2.4. Classificação Clara de Erros (Permanente vs. Temporário)
- A página de erro deve exibir visualmente o tipo de ocorrência:
  - **Temporário (`is_transient = true`):**
    - Timeouts (`is_timeout()`), falhas de conexão de rede (`is_connect()`) e erros de servidor na origem (502, 503, 504).
    - Rótulo amigável: `Instabilidade Temporária`.
    - Mensagem orientando a leitora a utilizar o botão "Tentar novamente".
  - **Permanente (`is_transient = false`):**
    - HTTP 403/401 (bloqueio/paywall na origem), HTTP 404/410 (conteúdo inexistente), caminhos ou URLs inválidas.
    - Rótulo amigável: `Bloqueio Definitivo` ou `Conteúdo Não Encontrado`.
    - Mensagem explicando que a tentativa imediata não alterará o resultado.
