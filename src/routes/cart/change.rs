//! POST /cart_items/{id}: change a line's quantity;
//! POST /cart_items/{id}/delete: remove it.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use super::streams;
use crate::AppState;
use crate::forms::FormData;
use crate::request::Req;
use crate::routes::Result;
use crate::views::{redirect, turbo};

pub async fn update(State(state): State<AppState>, req: Req, Path(id): Path<String>, form: FormData) -> Result {
    let Some(quantity) = form.get("quantity") else {
        return Ok((StatusCode::BAD_REQUEST, "quantity is missing").into_response());
    };
    let quantity = quantity.trim().parse::<i64>().unwrap_or(0);
    req.session.update(|data| data.cart.update_quantity(&id, quantity));
    answer(&state, &req).await
}

pub async fn destroy(State(state): State<AppState>, req: Req, Path(id): Path<String>, _form: FormData) -> Result {
    req.session.update(|data| data.cart.remove(&id));
    answer(&state, &req).await
}

/// The new cart table and widget, or back to the cart page.
async fn answer(state: &AppState, req: &Req) -> Result {
    if !req.wants_turbo_stream() {
        return Ok(redirect::to(&req.path_for("/kosik")));
    }
    let streams = vec![streams::contents(state, req).await?, streams::widget(state, req).await?];
    Ok(turbo::response(streams).into_response())
}
