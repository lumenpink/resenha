# Spec: Correção de Extração de Artigos do G1 Truncados (#10)

## Contexto
Ao processar artigos jornalísticos do portal G1 (`g1.globo.com`), o `resenha` recuperava apenas um único parágrafo do artigo (normalmente a citação com mais pontuações). 
A investigação da causa raiz revelou que:
1. O G1 fragmenta o corpo da notícia em múltiplos containers `<div id="chunk-XXXX"><div class="mc-column content-text active-extra-styles"><p>...</p></div></div>`.
2. A maior parte desses chunks reside dentro de uma div de paywall `<div class="wall protected-content">`.
3. O algoritmo da crate `readability` acumula o content score de cada parágrafo apenas em seu nó pai (`id.parent()`) e avô (`id.parent().parent()`). Com a tripla camada de wrappers em cada parágrafo, nenhum container ancestral comum (como `<article>` ou `div.wall`) recebe a soma cumulativa dos parágrafos, fazendo com que cada parágrafo fique isolado e apenas o fragmento com maior pontuação individual seja selecionado como o artigo completo.

## Objetivo
Aprimorar a etapa de pré-processamento `preprocess_html` em `src/extractor.rs` para:
1. Neutralizar e desenvelopar os wrappers de chunks (`div[id^="chunk-"]`) e wrappers de coluna de texto (`div.content-text`).
2. Neutralizar divs de paywall (`div.wall`, `div.protected-content`), convertendo-as para `div.article-body`.
3. Garantir que os parágrafos residam diretamente sob o container principal do artigo, permitindo que a pontuação do Readability agregue todos os parágrafos do texto da notícia.

## Critérios de Aceitação
1. Extração de artigos do G1 com múltiplos parágrafos preserva o artigo integral (> 10 parágrafos quando o original possuir essa extensão).
2. Títulos, figuras e imagens válidas do artigo continuam sendo extraídos e sanitizados sem quebras.
3. Teste automatizado com fixture real do G1 validando a extração completa e ausência de truncamento.
