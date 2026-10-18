use std::net::SocketAddr;

use anyhow::Context;
use piladerflice::{AppState, Config, app, commands};
use tracing_subscriber::EnvFilter;

/// Settings come from the environment (and, on a development machine, the
/// `.env` file):
/// - `HOST` / `PORT`: where to listen (default 127.0.0.1:4000, behind a reverse proxy)
/// - `APP_ENV=production`: load the assets and the WebAssembly once instead of watching them
/// - `APP_ROOT`: the release folder (translations, assets, pkg/, public/)
/// - `RUST_LOG`: log detail, e.g. `info` or `piladerflice=debug`
/// - `DATABASE_URL`: the SQLite database (default: `storage/development.sqlite3`)
/// - `STORAGE_DIR`: uploaded images (default: `storage/`)
/// - `SECRET_KEY_BASE`: encrypts the session cookie (required in production)
/// - `ORS_API_KEY`: openrouteservice, for the delivery price
/// - `SMTP_*`, `MAIL_FROM_ADDRESS`, `ORDER_NOTIFICATION_EMAIL`...: see `mail/settings.rs`
///
/// `piladerflice <command>` runs a command instead of the server: `setup`,
/// `migrate`, `admin`, `backup <file>` (see `commands/`).
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let development = std::env::var("APP_ENV").map_or(cfg!(debug_assertions), |env| env != "production");
    if development {
        // Settings that already exist win over the file.
        let _ = dotenvy::dotenv();
    }
    let config = Config::from_env(development)?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    if commands::run(&args, &config).await? {
        return Ok(());
    }

    let state = AppState::new(config).await?;
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "4000".into());
    let addr: SocketAddr = format!("{host}:{port}").parse().context("HOST/PORT")?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(
        "listening on http://{addr} ({})",
        if development { "development" } else { "production" }
    );
    axum::serve(listener, tower::make::Shared::new(app(state)))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    tracing::info!("stopped");
    Ok(())
}

/// Ctrl+C, or SIGTERM from systemd: requests being answered are finished
/// first, new connections are no longer taken.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::warn!("cannot listen for SIGTERM: {error}");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
    tracing::info!("shutting down");
}
