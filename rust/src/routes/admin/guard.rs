//! Only admins get into /admin. Runs before every admin page and form, so
//! no handler can forget the check: visitors who aren't signed in are sent
//! to the sign-in page (and come back afterwards), customers to the home
//! page with a message.

use axum::extract::{FromRequestParts, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::AppState;
use crate::request::Req;
use crate::session::Flash;
use crate::views::redirect;

pub async fn require_admin(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let (mut parts, body) = request.into_parts();
    let req = match Req::from_request_parts(&mut parts, &state).await {
        Ok(req) => req,
        Err(error) => return error.into_response(),
    };
    match &req.user {
        Some(current) if current.user.is_admin() => next.run(Request::from_parts(parts, body)).await,
        Some(_) => {
            req.session
                .set_flash(Flash::Alert("Nemáte oprávnění k přístupu do administrace.".into()));
            redirect::to("/")
        }
        None => {
            let here = match &req.query {
                Some(query) => format!("{}?{query}", req.path),
                None => req.path.clone(),
            };
            req.session.update(|data| data.return_to = Some(here));
            redirect::to("/prihlaseni")
        }
    }
}
