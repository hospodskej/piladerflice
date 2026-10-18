//! POST /cart_items/{id}: change a line's quantity;
//! POST /cart_items/{id}/delete: remove it. The cart page's island does the
//! same through server functions; these are the plain forms.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::forms::FormData;
use crate::request::Req;
use crate::routes::Result;
use crate::views::redirect;

pub async fn update(State(_state): State<AppState>, req: Req, Path(id): Path<String>, form: FormData) -> Result {
    let Some(quantity) = form.get("quantity") else {
        return Ok((StatusCode::BAD_REQUEST, "quantity is missing").into_response());
    };
    let quantity = quantity.trim().parse::<i64>().unwrap_or(0);
    req.session.update(|data| data.cart.update_quantity(&id, quantity));
    Ok(redirect::to(&req.path_for("/kosik")))
}

pub async fn destroy(State(_state): State<AppState>, req: Req, Path(id): Path<String>, _form: FormData) -> Result {
    req.session.update(|data| data.cart.remove(&id));
    Ok(redirect::to(&req.path_for("/kosik")))
}
