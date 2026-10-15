//! /prihlaseni: the sign-in page and form.

use axum::extract::State;
use axum::http::StatusCode;
use minijinja::context;

use crate::AppState;
use crate::accounts::password;
use crate::forms::FormData;
use crate::models::user;
use crate::request::Req;
use crate::routes::Result;
use crate::session::Flash;
use crate::views::{self, Page, redirect};

/// GET: the form. `?return_to=/kosik/udaje` comes back there after signing
/// in (only paths on this site).
pub async fn new(State(state): State<AppState>, req: Req) -> Result {
    if let Some(return_to) = req.query_param("return_to").filter(|path| is_local(path)) {
        req.session.update(|data| data.return_to = Some(return_to));
    }
    render(&state, &req, None).await
}

pub async fn create(State(state): State<AppState>, req: Req, form: FormData) -> Result {
    let email = form.get("email_address").unwrap_or_default();
    let given = form.get("password").unwrap_or_default();
    let user = user::find_by_email(&state.db, email).await?;
    // Same answer (and similar time) whether the e-mail exists or not.
    let user = match user {
        Some(user) if password::verify(given, &user.password_digest) => user,
        Some(_) => return failed(&state, &req).await,
        None => {
            password::verify(given, DUMMY_DIGEST);
            return failed(&state, &req).await;
        }
    };
    super::sign_in(&state, &req, user.id).await?;
    let target = req
        .session
        .update(|data| data.return_to.take())
        .unwrap_or_else(|| req.path_for("/muj-ucet"));
    Ok(redirect::to(&target))
}

/// A bcrypt hash of nothing in particular, checked when the e-mail is unknown.
const DUMMY_DIGEST: &str = "$2a$12$NxBaAjc.5tabNjXY/rmokeQN37LnflU0ZEdgmXV5797bm6Utrw0ua";

async fn failed(state: &AppState, req: &Req) -> Result {
    // The Rails app says this in Czech in both languages.
    let mut response = render(state, req, Some(Flash::Alert("Nesprávný e-mail nebo heslo.".into()))).await?;
    *response.status_mut() = StatusCode::UNPROCESSABLE_ENTITY;
    Ok(response)
}

async fn render(state: &AppState, req: &Req, flash: Option<Flash>) -> Result {
    let mut page = Page::meta(state, req, "login");
    page.flash_now = flash;
    Ok(views::render(state, req, "sessions/new.html", page, context! {}).await)
}

/// Only paths of this site ("/x", not "//evil.example" or "https://...").
pub fn is_local(path: &str) -> bool {
    path.starts_with('/') && !path.starts_with("//") && !path.starts_with("/\\")
}

#[cfg(test)]
mod tests {
    #[test]
    fn return_to_must_stay_on_the_site() {
        assert!(super::is_local("/kosik/udaje"));
        assert!(!super::is_local("//evil.example"));
        assert!(!super::is_local("/\\evil.example"));
        assert!(!super::is_local("https://evil.example"));
    }
}
