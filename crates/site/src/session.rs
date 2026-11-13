//! The browser session (a cookie with a random id; the data stays in SQLite):
//! the cart, checkout progress, delivery estimate, sign-in and flash messages.

use crate::i18n::Locale;
use crate::state::AppState;
use crate::visitor::{Flash, SignedInUser, Theme, Visitor};
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use domain::cart::Cart;
use domain::delivery::DeliveryEstimate;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use tower_sessions::Session;

const CART: &str = "cart";
const DELIVERY: &str = "delivery_estimate";
const FLASH: &str = "flash";
const SIGN_IN: &str = "sign_in";
pub const RETURN_TO: &str = "return_to";

/// Admins sign in again after this; customers stay signed in.
pub const ADMIN_SESSION_LIFETIME: Duration = Duration::days(30);

/// Who signed in this browser, and when.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct SignIn {
    user_id: i64,
    /// The user's `session_epoch` at sign-in; a password change raises it,
    /// which signs out every browser.
    epoch: i64,
    #[serde(with = "time::serde::rfc3339")]
    at: OffsetDateTime,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error(transparent)]
    Session(#[from] tower_sessions::session::Error),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

pub async fn cart(session: &Session) -> Cart {
    session.get(CART).await.ok().flatten().unwrap_or_default()
}

pub async fn save_cart(session: &Session, cart: &Cart) -> Result<(), SessionError> {
    Ok(session.insert(CART, cart).await?)
}

pub async fn delivery_estimate(session: &Session) -> Option<DeliveryEstimate> {
    session.get(DELIVERY).await.ok().flatten()
}

pub async fn save_delivery_estimate(session: &Session, estimate: Option<&DeliveryEstimate>) -> Result<(), SessionError> {
    match estimate {
        Some(e) => session.insert(DELIVERY, e).await?,
        None => {
            session.remove::<DeliveryEstimate>(DELIVERY).await?;
        }
    }
    Ok(())
}

pub async fn flash(session: &Session, flash: Flash) -> Result<(), SessionError> {
    Ok(session.insert(FLASH, flash).await?)
}

/// Signs `user_id` in. The session gets a new id (so one planted before
/// sign-in is useless) but keeps the cart and everything else.
pub async fn sign_in(session: &Session, user_id: i64, epoch: i64) -> Result<(), SessionError> {
    session.cycle_id().await?;
    session.insert(SIGN_IN, SignIn { user_id, epoch, at: OffsetDateTime::now_utc() }).await?;
    Ok(())
}

/// Signs out and forgets everything in the session.
pub async fn sign_out(session: &Session) -> Result<(), SessionError> {
    Ok(session.flush().await?)
}

/// The signed-in user, if the sign-in is still valid.
async fn signed_in_user(session: &Session, state: &AppState) -> Result<Option<SignedInUser>, SessionError> {
    let Some(sign_in) = session.get::<SignIn>(SIGN_IN).await? else {
        return Ok(None);
    };
    let row = sqlx::query!(
        r#"SELECT id AS "id!", email_address, first_name, role, theme, avatar_path, session_epoch FROM users WHERE id = ?"#,
        sign_in.user_id
    )
    .fetch_optional(&state.db)
    .await?;

    let valid = row.as_ref().is_some_and(|u| {
        u.session_epoch == sign_in.epoch && (u.role != "admin" || sign_in.at + ADMIN_SESSION_LIFETIME > OffsetDateTime::now_utc())
    });
    if !valid {
        session.remove::<SignIn>(SIGN_IN).await?;
        return Ok(None);
    }
    Ok(row.map(|u| SignedInUser {
        id: u.id,
        email: u.email_address,
        first_name: u.first_name,
        admin: u.role == "admin",
        theme: u.theme.as_deref().and_then(Theme::parse),
        avatar: u.avatar_path,
    }))
}

/// Axum middleware: puts the [`Visitor`] into the request for the pages.
pub async fn load_visitor(State(state): State<AppState>, session: Session, mut request: Request, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let user = match signed_in_user(&session, &state).await {
        Ok(user) => user,
        Err(e) => {
            tracing::warn!("could not load the signed-in user: {e}");
            None
        }
    };
    let visitor = Visitor {
        locale: Locale::of_path(&path),
        query: request.uri().query().map(str::to_owned),
        path,
        user,
        cart: cart(&session).await,
        delivery: delivery_estimate(&session).await,
        flash: session.remove::<Flash>(FLASH).await.ok().flatten(),
    };
    request.extensions_mut().insert(visitor);
    next.run(request).await
}
