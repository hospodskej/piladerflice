//! The admin area (/admin), Czech only. Every page and form is behind
//! [`guard::require_admin`]. One file per section; what the records are and
//! how they are checked lives in `src/admin/`. Forms post to the record's
//! URL to save it, and to `.../delete` to delete it.

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
use axum::middleware;
use axum::routing::{get, post};
use chrono::NaiveDateTime;

use crate::AppState;
use crate::admin::images::{self, Image};
use crate::czech_time;
use crate::forms::{Errors, MAX_ADMIN_UPLOAD_BYTES, UploadForm};
use crate::money::delimit;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(dashboard::index))
        .route("/orders", get(orders::index))
        .route("/orders/{id}", get(orders::show))
        .route("/orders/{id}/delete", post(orders::delete))
        .route("/inquiries", get(inquiries::index))
        .route("/inquiries/{id}", get(inquiries::show))
        .route("/catalog_products", get(products::index).post(products::create))
        .route("/catalog_products/new", get(products::new))
        .route("/catalog_products/{id}/edit", get(products::edit))
        .route("/catalog_products/{id}", post(products::update))
        .route("/catalog_products/{id}/delete", post(products::delete))
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
            post(variants::update),
        )
        .route(
            "/catalog_products/{product_id}/catalog_variants/{id}/delete",
            post(variants::delete),
        )
        .route("/promos", get(promos::index).post(promos::create))
        .route("/promos/new", get(promos::new))
        .route("/promos/{id}/edit", get(promos::edit))
        .route("/promos/{id}", post(promos::update))
        .route("/promos/{id}/delete", post(promos::delete))
        .route("/pricelist_items", get(pricelist::index).post(pricelist::create))
        .route("/pricelist_items/{id}", post(pricelist::update))
        .route("/pricelist_items/{id}/delete", post(pricelist::delete))
        .route("/pricelist_items/{id}/move_up", post(pricelist::move_up))
        .route("/pricelist_items/{id}/move_down", post(pricelist::move_down))
        .route("/inquiry_form_options", get(options::index).post(options::create))
        .route("/inquiry_form_options/{id}", post(options::update))
        .route("/inquiry_form_options/{id}/delete", post(options::delete))
        .route("/inquiry_form_options/{id}/move_up", post(options::move_up))
        .route("/inquiry_form_options/{id}/move_down", post(options::move_down))
        .layer(DefaultBodyLimit::max(MAX_ADMIN_UPLOAD_BYTES))
        .route_layer(middleware::from_fn_with_state(state, guard::require_admin))
}

/// The photo chosen in a form's `field`, checked; None when none was
/// chosen. A file that isn't a usable picture becomes an error of the form.
async fn chosen_image(form: &UploadForm, field: &str, errors: &mut Errors) -> Option<(Image, String)> {
    match images::from_form(form, field).await {
        Ok(image) => image,
        Err(_) => {
            errors.add("image", images::NOT_AN_IMAGE);
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
