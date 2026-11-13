//! The cart's forms: add, change a quantity, remove, discount code,
//! delivery estimate. Each redirects back (to the cart, or to the page it
//! came from).

use super::{field, locale_of, Fields};
use crate::i18n::t;
use crate::state::AppState;
use crate::visitor::Visitor;
use axum::Extension;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, Uri};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Form;
use tower_sessions::Session;

/// The page the form was on, if it's on this site; else `fallback`.
fn back(headers: &HeaderMap, fallback: String) -> Response {
    let referer = headers.get(header::REFERER).and_then(|r| r.to_str().ok()).and_then(|r| {
        let host = headers.get(header::HOST)?.to_str().ok()?;
        let path = r.split_once(host).map(|(_, path)| path)?;
        (path.starts_with('/') && !path.starts_with("//")).then(|| path.to_owned())
    });
    Redirect::to(&referer.unwrap_or(fallback)).into_response()
}

/// A one-off note on the cart page about the discount code: (text, is an error).
pub const DISCOUNT_NOTE: &str = "discount_note";
/// Why the last delivery estimate failed (for the "Kam vozíme?" box).
pub const DELIVERY_ERROR: &str = "delivery_error";

pub async fn add(State(state): State<AppState>, session: Session, Extension(mut v): Extension<Visitor>, uri: Uri, headers: HeaderMap, Form(fields): Form<Fields>) -> Response {
    let l = locale_of(&uri);
    let id = field(&fields, "catalog_variant_id").and_then(|id| id.parse().ok());
    let quantity = field(&fields, "quantity").and_then(|q| q.parse().ok()).unwrap_or(1);
    if let Some(id) = id {
        if let Err(e) = crate::cart_ops::add(&state, &session, &mut v, id, quantity).await {
            tracing::info!("not added to the cart: {e}");
        }
    }
    back(&headers, l.path("/"))
}

pub async fn update(session: Session, Extension(mut v): Extension<Visitor>, uri: Uri, Path(id): Path<i64>, Form(fields): Form<Fields>) -> Response {
    let l = locale_of(&uri);
    let quantity = field(&fields, "quantity").and_then(|q| q.parse::<i64>().ok()).unwrap_or(1).clamp(0, i64::from(u32::MAX));
    if let Err(e) = crate::cart_ops::set_quantity(&session, &mut v, id, u32::try_from(quantity).unwrap_or(0)).await {
        tracing::warn!("quantity not changed: {e}");
    }
    Redirect::to(&l.path("/kosik")).into_response()
}

pub async fn remove(session: Session, Extension(mut v): Extension<Visitor>, uri: Uri, Path(id): Path<i64>) -> Response {
    let l = locale_of(&uri);
    if let Err(e) = crate::cart_ops::set_quantity(&session, &mut v, id, 0).await {
        tracing::warn!("not removed: {e}");
    }
    Redirect::to(&l.path("/kosik")).into_response()
}

pub async fn apply_discount(
    State(state): State<AppState>,
    session: Session,
    Extension(mut v): Extension<Visitor>,
    ip: refractory::RemoteIp,
    uri: Uri,
    Form(fields): Form<Fields>,
) -> Response {
    let l = locale_of(&uri);
    if let Some(refused) = super::too_many_attempts(&state, &session, &ip, &crate::limits::DISCOUNT_CODE, l, "/kosik").await {
        return refused;
    }
    let note = match field(&fields, "code") {
        None => (t::cart::discount_errors::blank(l).to_owned(), true),
        Some(code) => match crate::cart_ops::apply_discount(&state, &session, &mut v, code).await {
            Ok(None) => (t::cart::discount_applied(l, v.cart.discount_code.as_deref().unwrap_or(code)), false),
            Ok(Some((reason, min_order))) => (discount_reason(&v, reason, min_order), true),
            Err(e) => {
                tracing::warn!("discount code not checked: {e}");
                (t::cart::discount_errors::not_found(l).to_owned(), true)
            }
        },
    };
    let _ = session.insert(DISCOUNT_NOTE, note).await;
    Redirect::to(&l.path("/kosik")).into_response()
}

/// Why a code can't be used, for the customer.
pub fn discount_reason(v: &crate::visitor::Visitor, reason: domain::discount::Unusable, min_order: Option<domain::money::Czk>) -> String {
    use domain::discount::Unusable;
    let l = v.locale;
    let amount = min_order.map(|m| v.shop.price(l, m)).unwrap_or_default();
    match reason {
        Unusable::NotFound => t::cart::discount_errors::not_found(l).to_owned(),
        Unusable::Inactive => t::cart::discount_errors::inactive(l).to_owned(),
        Unusable::Expired => t::cart::discount_errors::expired(l).to_owned(),
        Unusable::UsedUp => t::cart::discount_errors::used_up(l).to_owned(),
        Unusable::MinOrder => t::cart::discount_errors::min_order(l, amount),
    }
}

pub async fn remove_discount(session: Session, Extension(mut v): Extension<Visitor>, uri: Uri) -> Response {
    let l = locale_of(&uri);
    if let Err(e) = crate::cart_ops::remove_discount(&session, &mut v).await {
        tracing::warn!("discount not removed: {e}");
    }
    Redirect::to(&l.path("/kosik")).into_response()
}

pub async fn estimate_delivery(
    State(state): State<AppState>,
    session: Session,
    ip: refractory::RemoteIp,
    uri: Uri,
    headers: HeaderMap,
    Form(fields): Form<Fields>,
) -> Response {
    let l = locale_of(&uri);
    if state.limiter.check(&crate::limits::DELIVERY_LOOKUP, ip.key()).await.is_err() {
        let _ = session.insert(DELIVERY_ERROR, t::security::too_many_attempts(l).to_owned()).await;
        return back(&headers, l.path("/eshop"));
    }
    let query = field(&fields, "query").map(|q| q.split_whitespace().collect::<Vec<_>>().join(" "));
    let country = domain::vat::Country::or_cz(field(&fields, "country"));
    let error = match query {
        None => Some(t::delivery_estimate::blank(l).to_owned()),
        Some(query) => match crate::routing::estimate(&state, &query, country).await {
            Some(estimate) => {
                let _ = crate::session::save_delivery_estimate(&session, Some(&estimate)).await;
                None
            }
            None => Some(t::delivery_estimate::not_found(l, &query)),
        },
    };
    if let Some(error) = error {
        let _ = session.insert(DELIVERY_ERROR, error).await;
    }
    back(&headers, l.path("/eshop"))
}

pub async fn clear_delivery(session: Session, uri: Uri, headers: HeaderMap) -> Response {
    let l = locale_of(&uri);
    let _ = crate::session::save_delivery_estimate(&session, None).await;
    back(&headers, l.path("/eshop"))
}
