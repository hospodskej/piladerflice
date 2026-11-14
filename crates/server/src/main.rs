//! The Pila Derflice web server.

#![recursion_limit = "512"]

use axum::{
    Router, middleware,
    routing::{get, post},
};
use cuticle::{Csp, Cuticle, Hsts, PermissionsPolicy, Source, XFrameOptions};
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use site::app::{App, shell};
use site::state::AppState;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::str::FromStr;
use time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer, cookie::SameSite};
use tower_sessions_sqlx_store::SqliteStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let production = std::env::var("APP_ENV").is_ok_and(|e| e == "production");

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://storage/development.db".into());
    let db = SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::from_str(&database_url)?
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .foreign_keys(true),
        )
        .await?;
    sqlx::migrate!("../../migrations").run(&db).await?;

    // One-off commands: `server admin:create_user`, `server db:seed`.
    match std::env::args().nth(1).as_deref() {
        Some("admin:create_user") => return tasks::create_admin(&db).await,
        Some("db:seed") => return tasks::seed(&db, production).await,
        Some(other) => {
            return Err(format!("unknown command {other:?} (admin:create_user, db:seed)").into());
        }
        None => tasks::seed_if_empty(&db).await?,
    }

    let sessions = SqliteStore::new(db.clone());
    sessions.migrate().await?;
    let state = AppState::new(db);

    let conf = get_configuration(None)?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let pkg_dir = std::path::Path::new(leptos_options.site_root.as_ref())
        .join(leptos_options.site_pkg_dir.as_ref());
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
                    leptos_axum::handle_server_fns_with_context(
                        move || provide_context(state.clone()),
                        request,
                    )
                }
            }),
        )
        .leptos_routes_with_context(&leptos_options, routes, context.clone(), {
            let options = leptos_options.clone();
            move || shell(options.clone())
        })
        // The pictures and the WASM bundle, or else the 404 page.
        .fallback(leptos_axum::file_and_error_handler_with_context(
            context, shell,
        ))
        .with_state(leptos_options)
        // The WASM bundle, answering "not modified" when the browser has it.
        .nest_service("/pkg", tower_http::services::ServeDir::new(pkg_dir))
        .merge(forms(state.clone()))
        .merge(site::plain::routes(state.clone()))
        // Photos uploaded in the admin: named after their content, so cached for a year.
        .nest_service(
            "/uploads",
            tower::ServiceBuilder::new()
                .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
                    axum::http::header::CACHE_CONTROL,
                    axum::http::HeaderValue::from_static("public, max-age=31536000, immutable"),
                ))
                .service(tower_http::services::ServeDir::new(
                    state.storage.join("uploads"),
                )),
        )
        .layer(middleware::from_fn(site::admin::handlers::admin_only))
        .layer(middleware::from_fn(static_cache))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            site::session::load_visitor,
        ))
        .layer(
            SessionManagerLayer::new(sessions)
                .with_name("piladerflice_session")
                .with_secure(production)
                .with_same_site(SameSite::Lax)
                .with_expiry(Expiry::OnInactivity(Duration::days(365))),
        )
        .layer(security_headers(production).into_layer())
        .layer(middleware::from_fn(same_origin_forms))
        .layer(tower_http::compression::CompressionLayer::new())
        .layer(middleware::from_fn(site::plain::redirects))
        // The site runs behind kamal-proxy, which says who the visitor is.
        .layer(axum::Extension(if production {
            refractory::ClientIp::behind_proxy()
        } else {
            refractory::ClientIp::direct()
        }));

    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}

/// The HTML forms' handlers (the same in both languages).
fn forms(state: AppState) -> Router {
    let both =
        |router: Router<AppState>, path: &str, handler: axum::routing::MethodRouter<AppState>| {
            router
                .route(path, handler.clone())
                .route(&format!("/at{path}"), handler)
        };
    let router = Router::new();
    let router = both(router, "/inquiries", post(site::forms::inquiry::create));
    let router = both(router, "/cart_items", post(site::forms::cart::add));
    let router = both(router, "/cart_items/{id}", post(site::forms::cart::update));
    let router = both(
        router,
        "/cart_items/{id}/odebrat",
        post(site::forms::cart::remove),
    );
    let router = both(
        router,
        "/kosik/slevovy-kod",
        post(site::forms::cart::apply_discount),
    );
    let router = both(
        router,
        "/kosik/slevovy-kod/zrusit",
        post(site::forms::cart::remove_discount),
    );
    let router = both(
        router,
        "/doprava-odhad",
        post(site::forms::cart::estimate_delivery),
    );
    let router = both(
        router,
        "/doprava-odhad/zrusit",
        post(site::forms::cart::clear_delivery),
    );
    let router = both(
        router,
        "/kosik/doprava",
        post(site::forms::checkout::update_shipping),
    );
    let router = both(
        router,
        "/kosik/udaje",
        post(site::forms::checkout::update_details),
    );
    let router = both(
        router,
        "/kosik/souhrn",
        post(site::forms::checkout::confirm),
    );
    let router = both(router, "/prihlaseni", post(site::forms::auth::sign_in));
    let router = both(router, "/odhlaseni", post(site::forms::auth::sign_out));
    let router = both(router, "/registrace", post(site::forms::auth::register));
    let router = both(
        router,
        "/zapomenute-heslo",
        post(site::forms::auth::request_reset),
    );
    let router = both(
        router,
        "/nove-heslo",
        post(site::forms::auth::set_new_password),
    );
    let router = both(
        router,
        "/nove-heslo/odkaz",
        get(site::forms::auth::open_reset_link),
    );
    let router = both(
        router,
        "/muj-ucet/upravit",
        post(site::forms::auth::update_account),
    );
    let router = both(
        router,
        "/muj-ucet/objednavky/{id}/znovu-objednat",
        post(site::forms::auth::reorder),
    );
    let router = both(
        router,
        "/muj-ucet/profilovy-obrazek",
        get(site::forms::auth::show_avatar)
            .post(site::forms::auth::upload_avatar)
            // Pictures over 2 MB are refused anyway; stop reading well before that.
            .layer(axum::extract::DefaultBodyLimit::max(3 * 1024 * 1024)),
    );
    let router = both(
        router,
        "/muj-ucet/profilovy-obrazek/smazat",
        post(site::forms::auth::remove_avatar),
    );
    router.merge(admin(state.clone())).with_state(state)
}

/// What the admin's forms post to (Czech only; admins only, see `admin_only`).
fn admin(state: AppState) -> Router<AppState> {
    use site::admin::handlers::{catalog, content, orders, settings};
    let _ = state;
    Router::new()
        .route("/admin/orders/{id}/smazat", post(orders::delete))
        .route("/admin/catalog_products", post(catalog::create_product))
        .route(
            "/admin/catalog_products/{id}",
            post(catalog::update_product),
        )
        .route(
            "/admin/catalog_products/{id}/smazat",
            post(catalog::delete_product),
        )
        .route(
            "/admin/catalog_products/{product_id}/catalog_variants",
            post(catalog::create_variant),
        )
        .route(
            "/admin/catalog_products/{product_id}/catalog_variants/{id}",
            post(catalog::update_variant),
        )
        .route(
            "/admin/catalog_products/{product_id}/catalog_variants/{id}/smazat",
            post(catalog::delete_variant),
        )
        .route("/admin/pricelist_items", post(content::create_price))
        .route("/admin/pricelist_items/{id}", post(content::update_price))
        .route(
            "/admin/pricelist_items/{id}/smazat",
            post(content::delete_price),
        )
        .route(
            "/admin/pricelist_items/{id}/{direction}",
            post(content::move_price),
        )
        .route("/admin/inquiry_form_options", post(content::create_option))
        .route(
            "/admin/inquiry_form_options/{id}",
            post(content::update_option),
        )
        .route(
            "/admin/inquiry_form_options/{id}/smazat",
            post(content::delete_option),
        )
        .route(
            "/admin/inquiry_form_options/{id}/{direction}",
            post(content::move_option),
        )
        .route("/admin/promos", post(content::create_promo))
        .route("/admin/promos/{id}", post(content::update_promo))
        .route("/admin/promos/{id}/smazat", post(content::delete_promo))
        .route("/admin/doprava", post(settings::update_delivery))
        .route("/admin/dph/{category}", post(settings::update_vat))
        .route("/admin/slevove-kody", post(settings::create_code))
        .route("/admin/slevove-kody/{id}", post(settings::update_code))
        .route(
            "/admin/slevove-kody/{id}/smazat",
            post(settings::delete_code),
        )
        // Photos are shrunk in the browser; this is the ceiling for the rest.
        .layer(axum::extract::DefaultBodyLimit::max(16 * 1024 * 1024))
}

/// The Content-Security-Policy and the other security headers: the browser
/// runs only the site's own scripts (and Google Analytics), so injected HTML
/// can't run a script or send data elsewhere.
fn security_headers(production: bool) -> Cuticle {
    // Where Google Analytics sends its measurements (www.google.com/g/collect
    // with consent mode).
    let google_analytics = || {
        [
            "https://*.google-analytics.com",
            "https://*.analytics.google.com",
            "https://*.googletagmanager.com",
            "https://www.google.com",
        ]
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
        .img_src(
            [Source::SelfOrigin, Source::Data, Source::Blob]
                .into_iter()
                .chain(google_analytics()),
        )
        // style="" attributes are used throughout; styles can't run code.
        .style_src([Source::SelfOrigin, Source::UnsafeInline])
        .font_src([Source::SelfOrigin, Source::Data])
        .frame_src([Source::host("https://www.google.com")]) // the map in the footer
        .object_src([Source::None])
        .base_uri([Source::SelfOrigin])
        .form_action([Source::SelfOrigin])
        .frame_ancestors([Source::SelfOrigin]);
    let permissions = [
        "camera",
        "microphone",
        "geolocation",
        "usb",
        "payment",
        "gyroscope",
        "magnetometer",
    ]
    .into_iter()
    .fold(PermissionsPolicy::new(), PermissionsPolicy::deny);
    Cuticle::new()
        .csp(csp)
        .permissions_policy(permissions)
        .x_frame_options(XFrameOptions::SameOrigin)
        // Rails' force_ssl: two years, with subdomains.
        .hsts(production.then(|| {
            Hsts::new(std::time::Duration::from_secs(2 * 365 * 24 * 60 * 60)).include_subdomains()
        }))
}

/// Forms are only accepted from the site's own pages: a browser says where
/// a form came from (Sec-Fetch-Site, Origin), and another site is refused.
async fn same_origin_forms(
    request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    use axum::http::{Method, StatusCode, header};
    use axum::response::IntoResponse;
    let safe = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    let headers = request.headers();
    let cross_site = headers
        .get("sec-fetch-site")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == "cross-site");
    let foreign_origin = match (headers.get(header::ORIGIN), headers.get(header::HOST)) {
        (Some(origin), Some(host)) => {
            let origin = origin.to_str().unwrap_or_default();
            let host = host.to_str().unwrap_or_default();
            origin != "null"
                && origin
                    .split_once("://")
                    .is_none_or(|(_, rest)| !rest.eq_ignore_ascii_case(host))
        }
        _ => false,
    };
    if !safe && (cross_site || foreign_origin) {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }
    next.run(request).await
}

/// How long browsers keep the site's own files: pictures a week; the WASM
/// bundle is checked each time (it isn't named after its content).
async fn static_cache(
    request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    let rule = match request.uri().path() {
        p if p.starts_with("/img/") => Some("public, max-age=604800"),
        p if p.starts_with("/pkg/") => Some("no-cache"),
        "/icon.svg" | "/icon.png" => Some("public, max-age=86400"),
        _ => None,
    };
    let mut response = next.run(request).await;
    if let (Some(rule), true) = (rule, response.status().is_success()) {
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static(rule),
        );
    }
    response
}

/// The bundled stylesheet. Its address carries a hash of the content, so it
/// can be cached for a year.
async fn stylesheet() -> impl axum::response::IntoResponse {
    (
        [
            (axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (
                axum::http::header::CACHE_CONTROL,
                "public, max-age=31536000, immutable",
            ),
        ],
        site::assets::STYLESHEET,
    )
}

mod tasks {
    //! Commands run by hand (or from GitHub Actions through Kamal).

    use sqlx::SqlitePool;
    use std::error::Error;

    /// Creates the admin, or resets its password, from ADMIN_EMAIL and
    /// ADMIN_PASSWORD. Fails (exit status 1) when it can't, so a workflow
    /// running it shows red.
    pub async fn create_admin(db: &SqlitePool) -> Result<(), Box<dyn Error>> {
        let email = std::env::var("ADMIN_EMAIL").map_err(|_| "Set ADMIN_EMAIL first.")?;
        let password = std::env::var("ADMIN_PASSWORD").map_err(|_| "Set ADMIN_PASSWORD first.")?;
        let email = thymus::normalize_email(&email);
        let password = thymus::Password::new(password)
            .map_err(|e| format!("Could not save admin user: {e}"))?;
        let hash = tokio::task::spawn_blocking(move || thymus::PasswordHash::new(&password))
            .await??
            .into_string();
        // A new password signs the account out everywhere.
        sqlx::query!(
            "INSERT INTO users (email_address, password_hash, role) VALUES (?, ?, 'admin')
             ON CONFLICT (email_address) DO UPDATE SET password_hash = excluded.password_hash, role = 'admin',
                 session_epoch = session_epoch + 1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
            email,
            hash
        )
        .execute(db)
        .await?;
        println!("Admin user ready: {email}");
        Ok(())
    }

    /// The very first start (an empty database): the site's starting content.
    pub async fn seed_if_empty(db: &SqlitePool) -> Result<(), Box<dyn Error>> {
        let empty = sqlx::query_scalar!(r#"SELECT NOT EXISTS (SELECT 1 FROM catalog_products) AND NOT EXISTS (SELECT 1 FROM products) AS "x!: bool""#)
            .fetch_one(db)
            .await?;
        if empty {
            tracing::info!("empty database: loading the starting content");
            seed(db, false).await?;
        }
        Ok(())
    }

    /// Loads seeds/seed.sql (the catalogue, price list, promos, texts). It
    /// replaces that content, so on a live site with content it only runs
    /// with FORCE_SEED=1.
    pub async fn seed(db: &SqlitePool, production: bool) -> Result<(), Box<dyn Error>> {
        let has_content = sqlx::query_scalar!(r#"SELECT EXISTS (SELECT 1 FROM catalog_products) OR EXISTS (SELECT 1 FROM products) AS "x!: bool""#)
            .fetch_one(db)
            .await?;
        if production && has_content && std::env::var("FORCE_SEED").as_deref() != Ok("1") {
            return Err("The production database already has content; refusing to re-seed. Set FORCE_SEED=1 to wipe it and seed anyway.".into());
        }
        let mut tx = db.begin().await?;
        for table in [
            "catalog_variants",
            "catalog_products",
            "products",
            "pricelist_items",
            "promos",
            "reviews",
            "services",
            "faq_items",
            "inquiry_form_options",
            "vat_rules",
        ] {
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!("DELETE FROM {table}")))
                .execute(&mut *tx)
                .await?;
        }
        sqlx::raw_sql(include_str!("../../../seeds/seed.sql"))
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        println!("Seeded.");
        Ok(())
    }
}
