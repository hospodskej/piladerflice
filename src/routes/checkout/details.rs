//! /kosik/udaje: name, e-mail, phone, billing address, and optionally a
//! company and a different delivery address.

use axum::extract::State;
use minijinja::context;

use super::guard::{self, Step};
use crate::AppState;
use crate::forms::FormData;
use crate::models::user::{PROFILE_FIELDS, User};
use crate::request::Req;
use crate::routes::Result;
use crate::views::{self, Page, redirect};

const FIELDS: [&str; 18] = [
    "first_name",
    "last_name",
    "email",
    "phone",
    "newsletter_opt_in",
    "billing_street",
    "billing_city",
    "billing_zip",
    "billing_country",
    "company_purchase",
    "company_name",
    "company_ico",
    "company_dic",
    "delivery_address_different",
    "delivery_street",
    "delivery_city",
    "delivery_zip",
    "delivery_country",
];

pub async fn show(State(state): State<AppState>, req: Req) -> Result {
    if let Some(back) = guard::check(&req, Step::Details) {
        return Ok(back);
    }
    if let Some(current) = &req.user {
        prefill_from_account(&req, &current.user);
    }
    let checkout = req.session.read(|data| data.checkout.fields().clone());
    let login_link = format!(
        "{}?{}",
        req.path_for("/prihlaseni"),
        form_urlencoded::Serializer::new(String::new())
            .append_pair("return_to", &req.path_for("/kosik/udaje"))
            .finish()
    );
    let page = Page::meta(&state, &req, "checkout_details");
    let current_user_email = req.user.as_ref().map(|current| current.user.email_address.clone());
    Ok(views::render(
        &state,
        &req,
        "checkout/details.html",
        page,
        context! { checkout, login_link, current_user_email },
    )
    .await)
}

pub async fn update(State(_state): State<AppState>, req: Req, form: FormData) -> Result {
    if let Some(back) = guard::check(&req, Step::Details) {
        return Ok(back);
    }
    let allowed: Vec<(String, String)> = form
        .pairs()
        .iter()
        .filter(|(name, _)| FIELDS.contains(&name.as_str()))
        .cloned()
        .collect();
    req.session.update(|data| data.checkout.update(&allowed));
    Ok(redirect::to(&req.path_for("/kosik/souhrn")))
}

/// A signed-in customer's saved details fill the form, unless this
/// checkout already has some.
fn prefill_from_account(req: &Req, user: &User) {
    if req.session.read(|data| data.checkout.get("first_name").is_some()) {
        return;
    }
    let mut fields: Vec<(String, String)> = PROFILE_FIELDS
        .iter()
        .filter_map(|name| user.field(name).map(|value| (name.to_string(), value)))
        .collect();
    fields.push(("email".into(), user.email_address.clone()));
    req.session.update(|data| data.checkout.update(&fields));
}
