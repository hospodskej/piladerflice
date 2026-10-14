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
/// - `APP_ROOT`: the release folder (templates, translations, assets, public/)
///
/// `piladerflice backup <file>` copies the database to a new file instead of
/// starting the server (safe while the server runs).
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let development = std::env::var("APP_ENV").map_or(cfg!(debug_assertions), |env| env != "production");
    let config = Config::from_env(development)?;

    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] => {}
        ["backup", target] => {
            let db = piladerflice::db::connect(&config.database_url).await?;
            piladerflice::db::backup(&db, std::path::Path::new(target)).await?;
            tracing::info!("database copied to {target}");
            return Ok(());
        }
        _ => anyhow::bail!("usage: piladerflice [backup <file>]"),
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
