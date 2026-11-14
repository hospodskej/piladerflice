//! Plain HTML form posts (they work without the WebAssembly too): each
//! handler does its work, leaves a one-off message and redirects.

pub mod auth;
pub mod cart;
pub mod checkout;
pub mod inquiry;

use crate::i18n::Locale;
use crate::visitor::Flash;
use axum::http::Uri;
use axum::response::{IntoResponse, Redirect, Response};
use tower_sessions::Session;

/// The language of the page that posted the form (/at/... is German).
pub fn locale_of(uri: &Uri) -> Locale {
    Locale::of_path(uri.path())
}

/// Leaves `flash` for the next page and redirects (303) to `path` in `locale`.
pub async fn redirect_with(
    session: &Session,
    locale: Locale,
    path: &str,
    flash: Option<Flash>,
) -> Response {
    if let Some(flash) = flash
        && let Err(e) = crate::session::flash(session, flash).await
    {
        tracing::warn!("could not keep the message: {e}");
    }
    Redirect::to(&locale.path(path)).into_response()
}

/// A form's fields as name/value pairs, repeated names kept in order.
pub type Fields = Vec<(String, String)>;

/// The first value of `name`, trimmed; `None` when missing or blank.
pub fn field<'a>(fields: &'a Fields, name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.trim())
        .filter(|v| !v.is_empty())
}

/// Every value of `name` (`items[palivove][druh][]`), in order.
pub fn all<'a>(fields: &'a Fields, name: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    fields
        .iter()
        .filter(move |(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

/// Leaves "too many attempts" and redirects to `path` when `rule` is
/// exhausted for this visitor's address; `None` when the attempt may go on.
pub async fn too_many_attempts(
    state: &crate::state::AppState,
    session: &Session,
    ip: &refractory::RemoteIp,
    rule: &refractory::Rule,
    locale: Locale,
    path: &str,
) -> Option<Response> {
    if state.limiter.check(rule, ip.key()).await.is_ok() {
        return None;
    }
    let message = crate::i18n::t::security::too_many_attempts(locale).to_owned();
    Some(redirect_with(session, locale, path, Some(Flash::Alert(message))).await)
}
