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
const CHECKOUT: &str = "checkout";
pub const LAST_ORDER: &str = "last_order_id";
pub const CHECKOUT_ERRORS: &str = "checkout_errors";
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
        r#"SELECT id AS "id!", email_address, first_name, last_name, phone, role, theme, avatar_path, session_epoch,
                  billing_street, billing_city, billing_zip, billing_country, company_purchase, company_name, company_ico, company_dic,
                  delivery_address_different, delivery_street, delivery_city, delivery_zip, delivery_country
           FROM users WHERE id = ?"#,
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
        profile: crate::visitor::Profile {
            first_name: u.first_name.clone(),
            last_name: u.last_name,
            phone: u.phone,
            billing: domain::checkout::PostalAddress {
                street: u.billing_street,
                city: u.billing_city.clone(),
                zip: u.billing_zip,
                country: u.billing_country.clone(),
            },
            company_purchase: u.company_purchase != 0,
            company_name: u.company_name,
            company_ico: u.company_ico,
            company_dic: u.company_dic,
            delivery_address_different: u.delivery_address_different != 0,
            delivery: domain::checkout::PostalAddress {
                street: u.delivery_street,
                city: u.delivery_city,
                zip: u.delivery_zip,
                country: u.delivery_country,
            },
        },
        first_name: u.first_name,
        admin: u.role == "admin",
        theme: u.theme.as_deref().and_then(Theme::parse),
        avatar: u.avatar_path,
        billing_city: u.billing_city,
        billing_country: u.billing_country,
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
    let shop = crate::shop::load(&state).await.unwrap_or_else(|e| {
        tracing::warn!("could not load the shop settings: {e}");
        crate::shop::Shop::default()
    });
    let cart = cart(&session).await;
    let discount = match &cart.discount_code {
        Some(code) => discount_code(&state.db, code).await.unwrap_or_else(|e| {
            tracing::warn!("could not load the discount code: {e}");
            None
        }),
        None => None,
    };
    let visitor = Visitor {
        shop,
        discount,
        locale: Locale::of_path(&path),
        query: request.uri().query().map(str::to_owned),
        path,
        user,
        cart,
        checkout: session.get(CHECKOUT).await.ok().flatten().unwrap_or_default(),
        delivery: delivery_estimate(&session).await,
        flash: session.remove::<Flash>(FLASH).await.ok().flatten(),
        registration_form: session.remove(crate::forms::auth::REGISTRATION_FORM).await.ok().flatten(),
        admin_refused: session.remove(crate::admin::handlers::REFUSED).await.ok().flatten(),
        checkout_errors: session.remove(CHECKOUT_ERRORS).await.ok().flatten().unwrap_or_default(),
        last_order: session.get(LAST_ORDER).await.ok().flatten(),
        discount_note: session.remove(crate::forms::cart::DISCOUNT_NOTE).await.ok().flatten(),
        delivery_error: session.remove(crate::forms::cart::DELIVERY_ERROR).await.ok().flatten(),
    };
    request.extensions_mut().insert(visitor);
    next.run(request).await
}

/// A discount code as saved in the admin (any letter case), or `None`.
pub async fn discount_code(db: &sqlx::SqlitePool, code: &str) -> Result<Option<domain::discount::DiscountCode>, sqlx::Error> {
    use domain::discount::{DiscountCode, DiscountKind};
    let code = code.trim();
    let Some(row) = sqlx::query!(
        "SELECT code, kind, value, active, valid_until, usage_limit, times_used, min_order_czk FROM discount_codes WHERE code = ? COLLATE NOCASE",
        code
    )
    .fetch_optional(db)
    .await?
    else {
        return Ok(None);
    };
    Ok(Some(DiscountCode {
        code: row.code,
        kind: row.kind.parse().unwrap_or(DiscountKind::Percent),
        value: row.value,
        active: row.active != 0,
        valid_until: row
            .valid_until
            .as_deref()
            .and_then(|d| time::Date::parse(d, &time::format_description::well_known::Iso8601::DATE).ok()),
        usage_limit: row.usage_limit,
        times_used: row.times_used,
        min_order: row.min_order_czk.map(domain::money::Czk),
    }))
}

pub async fn save_checkout(session: &Session, checkout: &domain::checkout::CheckoutState) -> Result<(), SessionError> {
    Ok(session.insert(CHECKOUT, checkout).await?)
}
