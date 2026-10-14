//! POST /cart_items: put a variant into the cart.

use axum::extract::State;
use axum::response::IntoResponse;
use minijinja::Value;

use super::streams;
use crate::AppState;
use crate::forms::FormData;
use crate::models::catalog;
use crate::models::upload::{self, Upload};
use crate::request::Req;
use crate::routes::{Result, not_found};
use crate::views::{cart_view, redirect, turbo};

pub async fn add(State(state): State<AppState>, req: Req, form: FormData) -> Result {
    let variant_id: i64 = form
        .get("catalog_variant_id")
        .and_then(|id| id.trim().parse().ok())
        .unwrap_or_default();
    let Some((product, variant)) = catalog::buyable_variant(&state.db, variant_id).await? else {
        return Ok(not_found::page(State(state), req).await);
    };
    // The variant's own photo, or the product's.
    let image = match upload::images_of(&state.db, "CatalogVariant").await?.get(&variant.id) {
        Some(own) => Some(own.url()),
        None => upload::images_of(&state.db, "CatalogProduct")
            .await?
            .get(&product.id)
            .map(Upload::url),
    };
    // Like Ruby's to_i: anything that isn't a number counts as 0, then 1.
    let quantity = form
        .get("quantity")
        .and_then(|q| q.trim().parse::<i64>().ok())
        .unwrap_or(0);
    let line = req.session.update(|data| {
        let id = data.cart.add(&product, &variant, image, quantity);
        data.cart.find(&id).cloned()
    });

    if !req.wants_turbo_stream() {
        let back = req
            .headers
            .get("referer")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        return Ok(redirect::to(&back.unwrap_or_else(|| req.path_for("/"))));
    }
    let mut response = vec![streams::widget(&state, &req).await?];
    if let Some(line) = line {
        let item = cart_view::line(&state, &req, &line).await?;
        response.push(streams::added_modal(&state, &req, Value::from_serialize(&item)).await?);
    }
    Ok(turbo::response(response).into_response())
}
