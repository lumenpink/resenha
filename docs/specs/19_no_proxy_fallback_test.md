# Spec: Declaração de no_proxy nos Testes de Fallback do Fetcher

- **Ticket**: #19
- **Tipo**: Fix / Testes e Integração Contínua
- **Data**: 2026-10-30

## 1. Problema e Motivação
Em ambientes de integração contínua (CI), a variável de ambiente `http_proxy` pode estar configurada para caches locais do sistema operacional (como cache APT).
Ao executar a suite de testes de fallback (`tests/fetcher_fallback_test.rs`), os servidores de teste sobem na interface de loopback (`127.0.0.1`).
Sem a declaração explícita de `no_proxy` nos testes, o cliente HTTP (`reqwest`) tenta enviar requisições locais para o proxy do sistema, que rejeita conexões em portas não autorizadas com `403 Configuration error`.

## 2. Requisitos e Contratos
1. **Escopo Cirúrgico:** Não alterar o ambiente global do runner nem o workflow de CI em larga escala.
2. **Isolamento de Teste:** Cada teste em `tests/fetcher_fallback_test.rs` deve configurar explicitamente sob `ENV_MUTEX`:
   - `std::env::set_var("no_proxy", "127.0.0.1,localhost");`
   - `std::env::set_var("NO_PROXY", "127.0.0.1,localhost");`
3. **Descontaminação do Ambiente:** Ao término da execução de cada teste, as variáveis são limpas ou restauradas ao estado original.

## 3. Critérios de Aceitação
- Testes de fallback em `tests/fetcher_fallback_test.rs` executam com sucesso mesmo na presença de `http_proxy` configurado no ambiente externo.
- `cargo test --test fetcher_fallback_test` 100% aprovado.
- `cccp check --strict` 100% conforme.
