# Especificação do Ticket #21: Bypass de CI em KOLLONTAI-018 e Hooks na Esteira

## 1. Contexto e Motivação
Na esteira remota de CI, o checkout limpo (`actions/checkout@v4`) não provisiona hooks locais em `.git/hooks/`, provocando falso positivo na regra `KOLLONTAI-018` durante `cccp check --strict`.

## 2. Requisitos da Correção
1. Atualizar `.gost/rules.toml` (`KOLLONTAI-018`) com detecção de ambiente hermético de CI (`CI`, `GITHUB_ACTIONS`, `FORGEJO_ACTIONS`).
2. Adicionar `cccp hook -i || true` imediatamente antes de `cccp check --strict` em `.forgejo/workflows/ci.yaml`.
3. Validar `cccp check --strict` com 100% de conformidade.

## 3. Árvore de Decisões (para a Natasha)
- **Defesa em profundidade:** Aplicamos o bypass na regra `KOLLONTAI-018` (alinhado ao perfil Kollontai #28) e a inicialização explícita de hooks no workflow para resiliência total.
