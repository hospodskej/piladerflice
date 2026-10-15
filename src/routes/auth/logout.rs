//! /odhlaseni: signing out (a DELETE, sent as a form with `_method=delete`).

use axum::extract::State;

use crate::AppState;
use crate::forms::FormData;
use crate::models::login_session;
use crate::request::Req;
use crate::routes::Result;
use crate::session::Flash;
use crate::views::redirect;

pub async fn destroy(State(state): State<AppState>, req: Req, _form: FormData) -> Result {
    if let Some(session_id) = crate::session::login_id(&req.session) {
        login_session::end(&state.db, session_id).await?;
    }
    req.session.set_login(None);
    req.session.set_flash(Flash::Notice("Byli jste odhlášeni.".into()));
    Ok(redirect::to(&req.path_for("/")))
}
