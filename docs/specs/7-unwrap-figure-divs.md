# Spec: Desenvelopamento de Divs Externas de Figuras e Imagens AMP

- **Ticket**: #7
- **Tipo**: Fix / Normalização de DOM
- **Data**: 2026-10-20

## Contexto e Problema
Portais de notícias modernos (como G1, Globo, UOL) organizam suas matérias em blocos ("chunks"), envelopando cada elemento editorial (inclusive figuras com imagens AMP) em contêineres como `<div id="chunk-..."> <div class="content-media"> <figure>...`.

O algoritmo Readability (`readability-rs`), durante a fase de limpeza da árvore DOM (`clean`), avalia todas as tags `<div>` com a função `scorer::is_useless`. Se `img_count > para_count + text_nodes_len`, e a div não possui texto/parágrafos diretos suficientes, a `<div>` inteira é julgada como inútil e descartada sumariamente junto com todas as suas imagens filhas.

## Solução
No pré-processamento de HTML (`preprocess_html` em `src/extractor.rs`):
1. Desenvelopar repetidamente qualquer `<div>` externa que envolva exclusivamente um elemento `<figure>`.
2. Desta forma, o `<figure>` fica exposto no nível do corpo do artigo (`<article>`), onde não sofre a penalidade nem o descarte de `is_useless` aplicado a `<div>`.
3. Garantir preservação das imagens e legendas no HTML sanitizado e no Markdown resultante.
