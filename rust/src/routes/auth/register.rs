//! /registrace: creating a customer account.

use axum::extract::State;
use axum::http::StatusCode;
use minijinja::context;

use crate::AppState;
use crate::accounts::password;
use crate::accounts::registration::{self, Registration};
use crate::forms::FormData;
use crate::models::user;
use crate::models::validation::error_count;
use crate::request::Req;
use crate::routes::Result;
use crate::session::Flash;
use crate::views::{self, Page, redirect};

pub async fn new(State(state): State<AppState>, req: Req) -> Result {
    render(&state, &req, "", Vec::new()).await
}

pub async fn create(State(state): State<AppState>, req: Req, form: FormData) -> Result {
    let email = form.get("user[email_address]").unwrap_or_default();
    let given = form.get("user[password]").unwrap_or_default();
    let email_taken = !email.trim().is_empty() && user::find_by_email(&state.db, email).await?.is_some();
    let errors = registration::errors(
        &Registration {
            email,
            password: given,
            confirmation: form.get("user[password_confirmation]"),
            email_taken,
        },
        &state.translations,
        req.locale,
    );
    if !errors.is_empty() {
        let mut response = render(&state, &req, email, errors).await?;
        *response.status_mut() = StatusCode::UNPROCESSABLE_ENTITY;
        return Ok(response);
    }

    let digest = password::hash(given)?;
    let user_id = user::create_customer(&state.db, email, &digest).await?;
    super::sign_in(&state, &req, user_id).await?;
    req.session.set_flash(Flash::Notice("Účet byl vytvořen.".into()));
    Ok(redirect::to(&req.path_for("/muj-ucet")))
}

async fn render(state: &AppState, req: &Req, email: &str, errors: Vec<String>) -> Result {
    let page = Page::meta(state, req, "register");
    let error_count = error_count(errors.len());
    Ok(views::render(
        state,
        req,
        "registrations/new.html",
        page,
        context! { email, errors, error_count },
    )
    .await)
}
