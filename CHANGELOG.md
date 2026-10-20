# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Pipeline soberano de Continuous Deployment no nó central (`obizero`) com healthcheck ativo de 15s, rollback automático e saneamento de camadas órfãs via `docker image prune -f` (closes #5).
- Descomissionamento definitivo e purga de gatilhos do Watchtower.
- Inicialização do repositório em conformidade com o padrão GOST.
