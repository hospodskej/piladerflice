//! /kosik/souhrn: everything once more, the total with delivery, and the
//! button that places the order (with the terms accepted).

use axum::extract::State;
use axum::http::StatusCode;
use minijinja::context;

use super::guard::{self, Step};
use super::order_draft::{self, Draft};
use crate::AppState;
use crate::forms::FormData;
use crate::mail;
use crate::request::Req;
use crate::routes::Result;
use crate::views::{self, Page, redirect};

pub async fn show(State(state): State<AppState>, req: Req) -> Result {
    if let Some(back) = guard::check(&req, Step::Summary) {
        return Ok(back);
    }
    let draft = order_draft::build(&state, &req).await?;
    render(&state, &req, &draft, Vec::new()).await
}

pub async fn confirm(State(state): State<AppState>, req: Req, form: FormData) -> Result {
    if let Some(back) = guard::check(&req, Step::Summary) {
        return Ok(back);
    }
    let draft = order_draft::build(&state, &req).await?;

    if form.get("terms_agreement") != Some("1") {
        let errors = vec![state.translations.text(req.locale, "checkout.terms_required", &[])];
        return render(&state, &req, &draft, errors).await;
    }
    let errors = draft.order.errors(&state.translations, req.locale);
    if !errors.is_empty() {
        return render(&state, &req, &draft, errors).await;
    }

    let order = draft.order.insert(&state.db).await?;
    // Phase 4 (accounts): link the order to the signed-in customer and save
    // the details to their account.
    for email in [
        mail::orders::new_order(&state, &order).await,
        mail::orders::customer_confirmation(&state, &order).await,
    ] {
        match email {
            Ok(email) => state.mailer.send(email),
            Err(error) => tracing::error!("e-mail for order #{} not built: {error:#}", order.id),
        }
    }
    req.session.update(|data| {
        data.cart.clear();
        data.checkout.clear();
        data.last_order_id = Some(order.id);
    });
    Ok(redirect::to(&req.path_for("/kosik/dekujeme")))
}

/// The summary page; with errors (terms not accepted, details missing) it
/// answers 422, so Turbo shows it in place of the page.
async fn render(state: &AppState, req: &Req, draft: &Draft, errors: Vec<String>) -> Result {
    let mut page = Page::meta(state, req, "checkout_summary");
    if !errors.is_empty() {
        page.status = Some(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let data = order_draft::view(state, req.locale, draft).await;
    Ok(views::render(state, req, "checkout/summary.html", page, context! { errors, ..data }).await)
}
