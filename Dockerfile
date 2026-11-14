# The Pila Derflice web server: one Rust binary plus its site files
# (the WebAssembly bundle and the pictures). Built and deployed by Kamal.

FROM docker.io/library/rust:1.100-slim-bookworm AS build
WORKDIR /app
RUN apt-get update -qq && \
    apt-get install --no-install-recommends -y pkg-config libssl-dev git && \
    rm -rf /var/lib/apt/lists /var/cache/apt/archives
RUN rustup target add wasm32-unknown-unknown && \
    cargo install cargo-leptos --version 0.3.12 --locked
COPY . .
# Queries are checked against .sqlx at compile time; no database needed.
ENV SQLX_OFFLINE=true
RUN cargo leptos build --release && \
    mkdir -p /out && \
    cp target/release/server /out/server && \
    cp -r target/site /out/site

FROM docker.io/library/debian:bookworm-slim
RUN apt-get update -qq && \
    apt-get install --no-install-recommends -y ca-certificates sqlite3 tzdata && \
    rm -rf /var/lib/apt/lists /var/cache/apt/archives && \
    groupadd --system --gid 1000 app && \
    useradd app --uid 1000 --gid 1000 --create-home --shell /bin/bash
WORKDIR /app
COPY --from=build /out/server /app/server
COPY --from=build /out/site /app/site
RUN mkdir -p /app/storage && chown app:app /app/storage
USER 1000:1000

ENV APP_ENV=production \
    TZ=Europe/Prague \
    RUST_LOG=info \
    STORAGE_DIR=/app/storage \
    DATABASE_URL=sqlite:///app/storage/production.db \
    LEPTOS_OUTPUT_NAME=piladerflice \
    LEPTOS_SITE_ROOT=/app/site \
    LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_ENV=PROD \
    LEPTOS_SITE_ADDR=0.0.0.0:3000

EXPOSE 3000
# Runs the migrations at start (and seeds an empty database).
CMD ["/app/server"]
