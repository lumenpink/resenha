# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Desenvelopamento de `<div>` externas e tratamento de URLs com parênteses (closes #7).
- Inlining soberano de imagens em base64 na leitura web e tela 404/502 tipográfica (closes #8).
- Promoção de figuras de destaque de cabeçalho em portais como O Globo (closes #8).

### Added
- Pipeline soberano de Continuous Deployment no nó central (`obizero`) com healthcheck ativo de 15s, rollback automático e saneamento de camadas órfãs via `docker image prune -f` (closes #5).
- Descomissionamento definitivo e purga de gatilhos do Watchtower.
- Inicialização do repositório em conformidade com o padrão GOST.
