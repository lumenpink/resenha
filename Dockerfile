# ==============================================================================
# Estágio 1: Compilação hermética em Rust
# ==============================================================================
FROM rust:1-slim-bookworm AS builder

LABEL maintainer="Lumen Pink <hi@lumen.pink>"

WORKDIR /app

# Cache inteligente de dependências
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release || true

# Compilação real da aplicação
COPY src ./src
RUN touch src/main.rs src/lib.rs && cargo build --release && strip target/release/resenha

# ==============================================================================
# Estágio 2: Runtime mínimo e seguro (Non-root)
# ==============================================================================
FROM debian:bookworm-slim AS runtime

LABEL org.opencontainers.image.title="resenha"
LABEL org.opencontainers.image.description="Subsistema de Extração, Sanitização e Destilação Tipográfica na Borda"
LABEL org.opencontainers.image.licenses="GPL-3.0"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Usuário de execução não-root
RUN useradd -m -u 1000 -U appuser

COPY --from=builder /app/target/release/resenha /usr/local/bin/resenha
RUN chmod +x /usr/local/bin/resenha

USER 1000:1000

ENV RESENHA_PORT=3030 \
    RESENHA_BIND=0.0.0.0 \
    RUST_LOG=info

EXPOSE 3030

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3030/healthz || exit 1

CMD ["/usr/local/bin/resenha", "serve"]
