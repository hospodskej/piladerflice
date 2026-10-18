//! Putting a variant into the cart: the add-to-cart form and the island's
//! server function do the same.

use crate::AppState;
use crate::models::catalog;
use crate::request::Req;
use crate::shop::cart_image;
use crate::shop::cart_line::CartLine;

/// Puts `quantity` of the variant into the session's cart; returns the
/// line, or None when the variant can't be bought.
pub async fn put_in_cart(
    state: &AppState,
    req: &Req,
    variant_id: i64,
    quantity: i64,
) -> anyhow::Result<Option<CartLine>> {
    let Some((product, variant)) = catalog::buyable_variant(&state.db, variant_id).await? else {
        return Ok(None);
    };
    let image = cart_image::for_variant(&state.db, &product, &variant).await?;
    Ok(req.session.update(|data| {
        let id = data.cart.add(&product, &variant, image, quantity);
        data.cart.find(&id).cloned()
    }))
}
