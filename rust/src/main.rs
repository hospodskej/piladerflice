use std::net::SocketAddr;

use anyhow::Context;
use piladerflice::{AppState, Config, app};
use tracing_subscriber::EnvFilter;

/// Settings come from the environment:
/// - `HOST` / `PORT`: where to listen (default 127.0.0.1:4000, behind a reverse proxy)
/// - `APP_ENV=production`: load templates and assets once instead of watching them
/// - `RUST_LOG`: log detail, e.g. `info` or `piladerflice=debug`
/// - `DATABASE_URL`: the SQLite database (default: the Rails development database)
/// - `STORAGE_DIR`: uploaded images (default: the Rails `storage/` folder)
/// - `SECRET_KEY_BASE`: encrypts the session cookie (required in production)
/// - `ORS_API_KEY`: openrouteservice, for the delivery price
/// - `SMTP_*`, `MAIL_FROM_ADDRESS`, `ORDER_NOTIFICATION_EMAIL`...: see `mail/settings.rs`
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let development = std::env::var("APP_ENV").map_or(cfg!(debug_assertions), |env| env != "production");
    let config = Config::from_env(development)?;
    let state = AppState::new(config).await?;

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "4000".into());
    let addr: SocketAddr = format!("{host}:{port}").parse().context("HOST/PORT")?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(
        "listening on http://{addr} ({})",
        if development { "development" } else { "production" }
    );
    axum::serve(listener, tower::make::Shared::new(app(state))).await?;
    Ok(())
}
