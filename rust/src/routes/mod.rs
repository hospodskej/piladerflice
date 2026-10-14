//! All URLs of the site, one module per page or area. Public pages answer in
//! both languages: `/kontakt` (Czech) and `/at/kontakt` (German) reach the
//! same handler, which reads the language from [`Req`](crate::request::Req).

mod account;
mod admin;
mod auth;
mod cart;
mod checkout;
mod components;
mod error;
mod eshop;
mod headers;
mod home;
mod inquiries;
mod kontakt;
mod legacy_locale;
mod legacy_storage;
mod legal;
pub(crate) mod not_found;
mod product_range;
mod public_files;
mod sitemap;
mod sluzby;
mod sortiment;
mod system;
mod uploads;

pub use error::{AppError, Result};

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::routing::{get, post};
use tower_http::trace::TraceLayer;

use crate::AppState;
use crate::assets;
use crate::forms::upload::MAX_UPLOAD_BYTES;

pub fn router(state: AppState) -> Router {
    Router::new()
        // Machines and files
        .route("/up", get(system::health))
        .route("/robots.txt", get(system::robots))
        .route("/sitemap.xml", get(sitemap::show))
        .route("/assets/{*path}", get(assets::serve))
        .route("/uploads/{key}/{filename}", get(uploads::serve))
        // Image addresses of the Rails app, sent on to /uploads
        .route(
            "/rails/active_storage/blobs/{signed_id}/{*filename}",
            get(legacy_storage::blob),
        )
        .route(
            "/rails/active_storage/blobs/redirect/{signed_id}/{*filename}",
            get(legacy_storage::blob),
        )
        .route(
            "/rails/active_storage/blobs/proxy/{signed_id}/{*filename}",
            get(legacy_storage::blob),
        )
        .route(
            "/rails/active_storage/representations/{signed_id}/{variation}/{*filename}",
            get(legacy_storage::representation),
        )
        .route(
            "/rails/active_storage/representations/redirect/{signed_id}/{variation}/{*filename}",
            get(legacy_storage::representation),
        )
        .route(
            "/rails/active_storage/representations/proxy/{signed_id}/{variation}/{*filename}",
            get(legacy_storage::representation),
        )
        // Pages
        .route("/", get(home::index))
        .route("/kontakt", get(kontakt::index))
        .route("/sluzby", get(sluzby::index))
        .route("/sortiment", get(sortiment::index))
        .route("/sortiment/palivove-drevo", product_range::page("palivove_drevo"))
        .route("/sortiment/stavebni-rezivo", product_range::page("stavebni_rezivo"))
        .route("/sortiment/truhlarske-rezivo", product_range::page("truhlarske_rezivo"))
        .route("/sortiment/okrasne-kamenivo", product_range::page("okrasne_kamenivo"))
        .route("/sortiment/vyrobni-zbytky", product_range::page("vyrobni_zbytky"))
        .route("/eshop", get(eshop::index))
        .route("/eshop/{key}", get(eshop::show))
        // Cart and checkout (Rails-style forms send PATCH/DELETE as POST + _method)
        .route("/kosik", get(cart::show))
        .route("/cart_items", post(cart::add))
        .route(
            "/cart_items/{id}",
            post(cart::change).patch(cart::update).delete(cart::destroy),
        )
        .route(
            "/kosik/doprava",
            get(checkout::shipping)
                .post(checkout::update_shipping)
                .patch(checkout::update_shipping),
        )
        .route(
            "/kosik/udaje",
            get(checkout::details)
                .post(checkout::update_details)
                .patch(checkout::update_details),
        )
        .route("/kosik/souhrn", get(checkout::summary).post(checkout::confirm))
        .route("/kosik/dekujeme", get(checkout::confirmation))
        .route("/inquiries", post(inquiries::create))
        // Accounts
        .route("/prihlaseni", get(auth::login_page).post(auth::login))
        .route("/odhlaseni", post(auth::logout).delete(auth::logout))
        .route("/registrace", get(auth::register_page).post(auth::register))
        .route("/muj-ucet", get(account::show))
        .route(
            "/muj-ucet/upravit",
            get(account::edit).post(account::update).patch(account::update),
        )
        .route(
            "/muj-ucet/profilovy-obrazek",
            get(account::avatar)
                .post(account::avatar_change)
                .patch(account::avatar_update)
                .delete(account::avatar_destroy)
                .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES)),
        )
        .route("/muj-ucet/objednavky/{id}/znovu-objednat", post(account::reorder))
        // Admin (Czech only, admins only)
        .nest("/admin", admin::router(state.clone()))
        // Legal
        .route("/obchodni-podminky", get(legal::terms))
        .route("/zasady-cookies", get(legal::cookies_policy))
        .route("/zasady-ochrany-osobnich-udaju", get(legal::privacy_policy))
        .fallback(public_files::or_not_found)
        .layer(middleware::from_fn(legacy_locale::redirect))
        .layer(middleware::from_fn_with_state(state.clone(), headers::add))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
