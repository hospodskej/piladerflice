//! POST /muj-ucet/objednavky/{id}/znovu-objednat: puts a past order's items
//! back into the cart (the ones still sold).

use axum::extract::{Path, State};

use super::require_login;
use crate::AppState;
use crate::forms::FormData;
use crate::models::catalog;
use crate::models::order::Order;
use crate::request::Req;
use crate::routes::{Result, not_found};
use crate::session::Flash;
use crate::shop::cart_image;
use crate::views::redirect;

pub async fn reorder(State(state): State<AppState>, req: Req, Path(id): Path<i64>, _form: FormData) -> Result {
    let current = match require_login(&state, &req) {
        Ok(current) => current,
        Err(redirect) => return Ok(*redirect),
    };
    let Some(order) = Order::find(&state.db, id)
        .await?
        .filter(|order| order.user_id == Some(current.user.id))
    else {
        return Ok(not_found::page(State(state), req).await);
    };

    let (mut added, mut skipped) = (0, 0);
    for item in order.items() {
        match catalog::buyable_variant(&state.db, item.catalog_variant_id).await? {
            Some((product, variant)) => {
                let image = cart_image::for_variant(&state.db, &product, &variant).await?;
                req.session
                    .update(|data| data.cart.add(&product, &variant, image, item.quantity));
                added += 1;
            }
            None => skipped += 1,
        }
    }

    let t = |key: &str, args: &[(&str, &str)]| state.translations.text(req.locale, key, args);
    if added == 0 {
        req.session.set_flash(Flash::Alert(t("auth.reorder_failed", &[])));
        return Ok(redirect::to(&req.path_for("/muj-ucet")));
    }
    let message = if skipped > 0 {
        t("auth.reorder_partial", &[("count", &skipped.to_string())])
    } else {
        t("auth.reorder_success", &[])
    };
    req.session.set_flash(Flash::Notice(message));
    Ok(redirect::to(&req.path_for("/kosik")))
}
