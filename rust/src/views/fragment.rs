//! A piece of a page (for a Turbo Stream), rendered with the same basic
//! variables a whole page has: language, form token and the cart.

use minijinja::{Value, context};

use super::cart_view;
use crate::AppState;
use crate::request::Req;

pub async fn render(state: &AppState, req: &Req, template: &str, data: Value) -> anyhow::Result<String> {
    let cart = cart_view::load(state, req).await?;
    let ctx = context! {
        locale => req.locale.code(),
        current_path => &req.plain_path,
        csrf_token => req.session.csrf_token(),
        cart,
        ..data
    };
    Ok(state.templates.render(template, ctx)?)
}
