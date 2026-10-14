//! The admin area (/admin), Czech only. Every page and form is behind
//! [`guard::require_admin`]. One file per section, like the Rails
//! controllers; what the records are and how they are checked lives in
//! `src/admin/`.

mod dashboard;
mod guard;
mod inquiries;
mod options;
mod orders;
mod params;
mod pricelist;
mod products;
mod promos;
mod variants;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::Method;
use axum::middleware;
use axum::routing::{get, post};
use chrono::NaiveDateTime;

use crate::AppState;
use crate::admin::images::{self, Image};
use crate::czech_time;
use crate::forms::upload::{MAX_ADMIN_UPLOAD_BYTES, UploadForm};
use crate::models::validation::Errors;
use crate::money::delimit;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(dashboard::index))
        .route("/orders", get(orders::index))
        .route(
            "/orders/{id}",
            get(orders::show).post(orders::change).delete(orders::change),
        )
        .route("/inquiries", get(inquiries::index))
        .route("/inquiries/{id}", get(inquiries::show))
        .route("/catalog_products", get(products::index).post(products::create))
        .route("/catalog_products/new", get(products::new))
        .route("/catalog_products/{id}/edit", get(products::edit))
        .route(
            "/catalog_products/{id}",
            post(products::change)
                .patch(products::change)
                .put(products::change)
                .delete(products::change),
        )
        .route(
            "/catalog_products/{product_id}/catalog_variants",
            post(variants::create),
        )
        .route(
            "/catalog_products/{product_id}/catalog_variants/new",
            get(variants::new),
        )
        .route(
            "/catalog_products/{product_id}/catalog_variants/{id}/edit",
            get(variants::edit),
        )
        .route(
            "/catalog_products/{product_id}/catalog_variants/{id}",
            post(variants::change)
                .patch(variants::change)
                .put(variants::change)
                .delete(variants::change),
        )
        .route("/promos", get(promos::index).post(promos::create))
        .route("/promos/new", get(promos::new))
        .route("/promos/{id}/edit", get(promos::edit))
        .route(
            "/promos/{id}",
            post(promos::change)
                .patch(promos::change)
                .put(promos::change)
                .delete(promos::change),
        )
        .route("/pricelist_items", get(pricelist::index).post(pricelist::create))
        .route(
            "/pricelist_items/{id}",
            post(pricelist::change)
                .patch(pricelist::change)
                .put(pricelist::change)
                .delete(pricelist::change),
        )
        .route(
            "/pricelist_items/{id}/move_up",
            post(pricelist::move_up).patch(pricelist::move_up),
        )
        .route(
            "/pricelist_items/{id}/move_down",
            post(pricelist::move_down).patch(pricelist::move_down),
        )
        .route("/inquiry_form_options", get(options::index).post(options::create))
        .route(
            "/inquiry_form_options/{id}",
            post(options::change)
                .patch(options::change)
                .put(options::change)
                .delete(options::change),
        )
        .route(
            "/inquiry_form_options/{id}/move_up",
            post(options::move_up).patch(options::move_up),
        )
        .route(
            "/inquiry_form_options/{id}/move_down",
            post(options::move_down).patch(options::move_down),
        )
        .layer(DefaultBodyLimit::max(MAX_ADMIN_UPLOAD_BYTES))
        .route_layer(middleware::from_fn_with_state(state, guard::require_admin))
}

/// What a form asks for: the HTTP method, or for a POST its Rails-style
/// `_method` ("patch", "delete").
fn method_of(http: &Method, form: &UploadForm) -> String {
    if http == Method::POST {
        form.method()
    } else {
        http.as_str().to_ascii_lowercase()
    }
}

/// The photo chosen in a form's `field`, checked; None when none was
/// chosen. A file that isn't a usable picture becomes an error of the form.
async fn chosen_image(form: &UploadForm, field: &str, errors: &mut Errors) -> Option<(Image, String)> {
    match images::from_form(form, field).await {
        Ok(image) => image,
        Err(_) => {
            errors.push("image", images::NOT_AN_IMAGE.into());
            None
        }
    }
}

/// "25 500 Kč" (the admin always shows crowns).
fn czk(amount: i64) -> String {
    format!("{} Kč", delimit(amount, " "))
}

/// "14.10.2026 18:28": a stored time in Czech time.
fn date(time: &NaiveDateTime) -> String {
    czech_time::from_utc(*time).format("%d.%m.%Y %H:%M").to_string()
}
