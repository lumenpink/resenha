# Especificação do Ticket #22: Delegar Reset de Container do Resenha ao Kombinat

## 1. Contexto e Motivação
O workflow `.forgejo/workflows/ci.yaml` executava SSH direto no nó `obizero`. Com o Kombinat orquestrando resets a partir do webhook da Gambi, a CI deve manter apenas build/push da imagem Docker para `gambi.2lp.in`.

## 2. Requisitos
1. Remover o step `Autonomous CD on obizero via SSH` de `.forgejo/workflows/ci.yaml`.
2. Alinhar `[services]` (`stack`, `node`) e `[sites]` (`dir = "site"`) em `.gost/delivery.toml`.
3. Atualizar `tests/audit_cd_workflow.sh` para auditar ausência de CD direto na CI.
