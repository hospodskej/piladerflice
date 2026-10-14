//! All URLs of the site, one module per page or area. Public pages answer in
//! both languages: `/kontakt` (Czech) and `/at/kontakt` (German) reach the
//! same handler, which reads the language from [`Req`](crate::request::Req).

mod cart;
mod checkout;
mod components;
mod error;
mod eshop;
mod home;
mod inquiries;
mod kontakt;
mod legacy_locale;
mod legal;
pub(crate) mod not_found;
mod product_range;
mod sitemap;
mod sluzby;
mod sortiment;
mod system;
mod uploads;

pub use error::{AppError, Result};

use axum::Router;
use axum::middleware;
use axum::routing::{get, post};
use tower_http::trace::TraceLayer;

use crate::AppState;
use crate::assets;

pub fn router(state: AppState) -> Router {
    Router::new()
        // Machines and files
        .route("/up", get(system::health))
        .route("/robots.txt", get(system::robots))
        .route("/sitemap.xml", get(sitemap::show))
        .route("/assets/{*path}", get(assets::serve))
        .route("/uploads/{key}/{filename}", get(uploads::serve))
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
        // Legal
        .route("/obchodni-podminky", get(legal::terms))
        .route("/zasady-cookies", get(legal::cookies_policy))
        .route("/zasady-ochrany-osobnich-udaju", get(legal::privacy_policy))
        .fallback(not_found::page)
        .layer(middleware::from_fn(legacy_locale::redirect))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
