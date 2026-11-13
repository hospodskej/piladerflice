//! The Pila Derflice web server.

use axum::{middleware, routing::get, Router};
use cuticle::{Csp, Cuticle, Hsts, Source};
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use site::app::{shell, App};
use site::state::AppState;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::str::FromStr;
use time::Duration;
use tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let production = std::env::var("APP_ENV").is_ok_and(|e| e == "production");

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://storage/development.db".into());
    let db = SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::from_str(&database_url)?
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .foreign_keys(true),
        )
        .await?;
    sqlx::migrate!("../../migrations").run(&db).await?;

    let sessions = SqliteStore::new(db.clone());
    sessions.migrate().await?;
    let state = AppState { db };

    let conf = get_configuration(None)?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        .route(site::assets::STYLESHEET_HREF, get(stylesheet))
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let state = state.clone();
                move || {
                    provide_context(state.clone());
                    cuticle::leptos::provide_nonce();
                }
            },
            {
                let options = leptos_options.clone();
                move || shell(options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        .layer(middleware::from_fn_with_state(state.clone(), site::session::load_visitor))
        .layer(
            SessionManagerLayer::new(sessions)
                .with_name("piladerflice_session")
                .with_secure(production)
                .with_same_site(SameSite::Lax)
                .with_expiry(Expiry::OnInactivity(Duration::days(365))),
        )
        .layer(security_headers(production).into_layer());

    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await?;
    Ok(())
}

/// The Content-Security-Policy and the other security headers. Only Google
/// Analytics and the footer's map come from outside.
fn security_headers(production: bool) -> Cuticle {
    let google_analytics = [
        Source::host("https://www.googletagmanager.com"),
        Source::host("https://*.google-analytics.com"),
        Source::host("https://*.analytics.google.com"),
    ];
    Cuticle::new()
        .csp(
            Csp::leptos()
                .extend("script-src", google_analytics.clone())
                .extend("connect-src", google_analytics.clone())
                .extend("img-src", [Source::Data, Source::Blob].into_iter().chain(google_analytics))
                .frame_src([Source::host("https://www.google.com")]),
        )
        .hsts(production.then(|| Hsts::new(std::time::Duration::from_secs(63_072_000)).include_subdomains()))
}

/// The bundled stylesheet. Its address carries a hash of the content, so it
/// can be cached for a year.
async fn stylesheet() -> impl axum::response::IntoResponse {
    (
        [
            (axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (axum::http::header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        site::assets::STYLESHEET,
    )
}
