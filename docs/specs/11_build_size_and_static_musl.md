# Especificação Técnica: Otimização de Binário e Linking Estático musl

- **Ticket**: #11
- **Escopo**: Configuração do perfil `[profile.release]` e flags de linking estático com musl libc no serviço `resenha`.

## 1. Contexto e Motivação
A ausência de configurações explícitas em `[profile.release]` resulta em binários Rust com mais de 70 MB no container e no host. Além disso, a compilação contra o target `*-unknown-linux-musl` sem `-C target-feature=+crt-static` gera dependência do interpretador dinâmico musl.

## 2. Requisitos Técnicos
1. `Cargo.toml`:
   - `opt-level = 3` (serviço web com foco em throughput de extração e parsing HTML).
   - `lto = "fat"`.
   - `codegen-units = 1`.
   - `panic = "abort"`.
   - `strip = true`.
2. `.cargo/config.toml`:
   - Configurar `rustflags = ["-C", "target-feature=+crt-static"]` para os targets `x86_64-unknown-linux-musl` e `aarch64-unknown-linux-musl`.
