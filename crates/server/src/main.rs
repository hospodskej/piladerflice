//! The Pila Derflice web server.

use axum::{middleware, routing::{get, post}, Router};
use cuticle::{Csp, Cuticle, Hsts, PermissionsPolicy, Source, XFrameOptions};
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
    let state = AppState::new(db);

    let conf = get_configuration(None)?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // What every page and server function can reach: the database and the
    // request's CSP nonce.
    let context = {
        let state = state.clone();
        move || {
            provide_context(state.clone());
            cuticle::leptos::provide_nonce();
        }
    };

    let app = Router::new()
        .route(site::assets::STYLESHEET_HREF, get(stylesheet))
        .route(
            "/api/{*fn_name}",
            axum::routing::post({
                let state = state.clone();
                move |request: axum::extract::Request| {
                    let state = state.clone();
                    leptos_axum::handle_server_fns_with_context(move || provide_context(state.clone()), request)
                }
            }),
        )
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            context.clone(),
            {
                let options = leptos_options.clone();
                move || shell(options.clone())
            },
        )
        // The pictures and the WASM bundle, or else the 404 page.
        .fallback(leptos_axum::file_and_error_handler_with_context(context, shell))
        .with_state(leptos_options)
        .merge(forms(state.clone()))
        .merge(site::plain::routes(state.clone()))
        .layer(middleware::from_fn_with_state(state.clone(), site::session::load_visitor))
        .layer(
            SessionManagerLayer::new(sessions)
                .with_name("piladerflice_session")
                .with_secure(production)
                .with_same_site(SameSite::Lax)
                .with_expiry(Expiry::OnInactivity(Duration::days(365))),
        )
        .layer(security_headers(production).into_layer())
        .layer(middleware::from_fn(site::plain::redirects))
        // The site runs behind kamal-proxy, which says who the visitor is.
        .layer(axum::Extension(if production {
            refractory::ClientIp::behind_proxy()
        } else {
            refractory::ClientIp::direct()
        }));

    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await?;
    Ok(())
}

/// The HTML forms' handlers (the same in both languages).
fn forms(state: AppState) -> Router {
    let both = |router: Router<AppState>, path: &str, handler: axum::routing::MethodRouter<AppState>| {
        router.route(path, handler.clone()).route(&format!("/at{path}"), handler)
    };
    let router = Router::new();
    let router = both(router, "/inquiries", post(site::forms::inquiry::create));
    let router = both(router, "/cart_items", post(site::forms::cart::add));
    let router = both(router, "/cart_items/{id}", post(site::forms::cart::update));
    let router = both(router, "/cart_items/{id}/odebrat", post(site::forms::cart::remove));
    let router = both(router, "/kosik/slevovy-kod", post(site::forms::cart::apply_discount));
    let router = both(router, "/kosik/slevovy-kod/zrusit", post(site::forms::cart::remove_discount));
    let router = both(router, "/doprava-odhad", post(site::forms::cart::estimate_delivery));
    let router = both(router, "/doprava-odhad/zrusit", post(site::forms::cart::clear_delivery));
    router.with_state(state)
}

/// The Content-Security-Policy and the other security headers: the browser
/// runs only the site's own scripts (and Google Analytics), so injected HTML
/// can't run a script or send data elsewhere.
fn security_headers(production: bool) -> Cuticle {
    // Where Google Analytics sends its measurements (www.google.com/g/collect
    // with consent mode).
    let google_analytics = || {
        ["https://*.google-analytics.com", "https://*.analytics.google.com", "https://*.googletagmanager.com", "https://www.google.com"]
            .map(Source::host)
    };
    let csp = Csp::new()
        .default_src([Source::SelfOrigin])
        // The few inline scripts (theme, Analytics, hydration) carry the
        // nonce; the islands are WebAssembly.
        .script_src([
            Source::SelfOrigin,
            Source::host("https://www.googletagmanager.com"),
            Source::Nonce,
            Source::WasmUnsafeEval,
        ])
        .connect_src([Source::SelfOrigin].into_iter().chain(google_analytics()))
        .img_src([Source::SelfOrigin, Source::Data, Source::Blob].into_iter().chain(google_analytics()))
        // style="" attributes are used throughout; styles can't run code.
        .style_src([Source::SelfOrigin, Source::UnsafeInline])
        .font_src([Source::SelfOrigin, Source::Data])
        .frame_src([Source::host("https://www.google.com")]) // the map in the footer
        .object_src([Source::None])
        .base_uri([Source::SelfOrigin])
        .form_action([Source::SelfOrigin])
        .frame_ancestors([Source::SelfOrigin]);
    let permissions = ["camera", "microphone", "geolocation", "usb", "payment", "gyroscope", "magnetometer"]
        .into_iter()
        .fold(PermissionsPolicy::new(), PermissionsPolicy::deny);
    Cuticle::new()
        .csp(csp)
        .permissions_policy(permissions)
        .x_frame_options(XFrameOptions::SameOrigin)
        // Rails' force_ssl: two years, with subdomains.
        .hsts(production.then(|| Hsts::new(std::time::Duration::from_secs(2 * 365 * 24 * 60 * 60)).include_subdomains()))
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
