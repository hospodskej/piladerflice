//! POST /cart_items: put a variant into the cart. The page's add-to-cart
//! island does the same through a server function (`islands/cart.rs`);
//! this is the plain form, before the island has come alive.

use axum::extract::State;

use crate::AppState;
use crate::forms::FormData;
use crate::request::Req;
use crate::routes::{Result, not_found};
use crate::shop::adding::put_in_cart;
use crate::views::redirect;

pub async fn add(State(state): State<AppState>, req: Req, form: FormData) -> Result {
    let variant_id: i64 = form
        .get("catalog_variant_id")
        .and_then(|id| id.trim().parse().ok())
        .unwrap_or_default();
    // Anything that isn't a number counts as 0, then 1.
    let quantity = form
        .get("quantity")
        .and_then(|q| q.trim().parse::<i64>().ok())
        .unwrap_or(0);
    if put_in_cart(&state, &req, variant_id, quantity).await?.is_none() {
        return Ok(not_found::page(State(state), req).await);
    }
    let back = req
        .headers
        .get("referer")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    Ok(redirect::to(&back.unwrap_or_else(|| req.path_for("/"))))
}
