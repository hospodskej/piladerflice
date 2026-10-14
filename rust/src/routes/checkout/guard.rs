//! Which checkout steps a request may see: none with an empty cart, the
//! details only after shipping is chosen, the summary only with complete
//! details. Otherwise the visitor is sent back to the step to do first.

use axum::response::Response;

use crate::request::Req;
use crate::views::redirect;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Step {
    Shipping,
    Details,
    Summary,
}

/// None when the step may be shown, else the redirect to the missing step.
pub fn check(req: &Req, step: Step) -> Option<Response> {
    let (empty, shipping_done, details_done) = req.session.read(|data| {
        (
            data.cart.is_empty(),
            data.checkout.shipping_selected(),
            data.checkout.details_complete(),
        )
    });
    if empty {
        return Some(redirect::to(&req.path_for("/kosik")));
    }
    if step >= Step::Details && !shipping_done {
        return Some(redirect::to(&req.path_for("/kosik/doprava")));
    }
    if step >= Step::Summary && !details_done {
        return Some(redirect::to(&req.path_for("/kosik/udaje")));
    }
    None
}
