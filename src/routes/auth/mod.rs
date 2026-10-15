//! Signing in, signing out and creating an account.

mod login;
mod logout;
mod register;

pub use login::{create as login, new as login_page};
pub use logout::destroy as logout;
pub use register::{create as register, new as register_page};

use crate::AppState;
use crate::models::login_session;
use crate::request::Req;
use crate::routes::Result;

/// Starts a login for `user_id` in this browser.
pub(super) async fn sign_in(state: &AppState, req: &Req, user_id: i64) -> Result<()> {
    let session_id = login_session::start(
        &state.db,
        user_id,
        req.client_ip().as_deref(),
        req.user_agent().as_deref(),
    )
    .await?;
    req.session.set_login(Some(session_id));
    Ok(())
}
