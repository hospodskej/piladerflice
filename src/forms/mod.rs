//! Submitted forms. [`FormData`] is what POST handlers take: it reads the
//! form fields, but only after checking the form protection token (and the
//! `Origin`), so a handler never sees a forged request.
//!
//! Fields are read like Rails params: `get("quantity")` (the last value when
//! a name repeats, so a hidden "0" before a checked checkbox gives "1"),
//! `all("delka[]")` for the repeated fields of the quote form.
//!
//! [`upload::UploadForm`] is the same for forms with a file (the profile
//! picture).

pub mod upload;

use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use crate::AppState;
use crate::request::Req;
use crate::session::csrf;

/// Forms are small; anything bigger is refused.
const MAX_FORM_BYTES: usize = 64 * 1024;

#[derive(Debug, Default, Clone)]
pub struct FormData {
    pairs: Vec<(String, String)>,
}

impl FormData {
    pub fn parse(body: &[u8]) -> Self {
        Self {
            pairs: form_urlencoded::parse(body).into_owned().collect(),
        }
    }

    /// The last value of a field (Rails' `params[:name]`).
    pub fn get(&self, name: &str) -> Option<&str> {
        self.pairs
            .iter()
            .rev()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// Every value of a repeated field, in order.
    pub fn all(&self, name: &str) -> Vec<&str> {
        self.pairs
            .iter()
            .filter(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
            .collect()
    }

    pub fn pairs(&self) -> &[(String, String)] {
        &self.pairs
    }

    /// The method a Rails-style form asks for with its hidden `_method`
    /// field ("patch", "delete"); "post" without one.
    pub fn method(&self) -> String {
        self.get("_method").unwrap_or("post").to_lowercase()
    }
}

/// Reading fields, whichever kind of form they came in.
pub trait Fields {
    fn field(&self, name: &str) -> Option<&str>;
}

impl Fields for FormData {
    fn field(&self, name: &str) -> Option<&str> {
        self.get(name)
    }
}

/// The form was refused: forged, from another site, or too big.
#[derive(Debug)]
pub struct Refused(pub StatusCode);

impl IntoResponse for Refused {
    fn into_response(self) -> Response {
        (self.0, Html(crate::views::error_pages::rejected())).into_response()
    }
}

/// Checks the Origin and the form token; the shared part of both form kinds.
pub(crate) fn verify(req: &Req, headers: &axum::http::HeaderMap, form_token: Option<&str>) -> Result<(), Refused> {
    let origin = headers.get("origin").and_then(|value| value.to_str().ok());
    if !csrf::origin_allowed(origin, &req.base_url) {
        tracing::warn!("form from another site refused (Origin {origin:?})");
        return Err(Refused(StatusCode::UNPROCESSABLE_ENTITY));
    }
    let header_token = headers.get("x-csrf-token").and_then(|value| value.to_str().ok());
    let given = form_token.or(header_token).unwrap_or_default();
    if !csrf::matches(&req.session.csrf_token(), given) {
        tracing::warn!("form without a valid authenticity token refused");
        return Err(Refused(StatusCode::UNPROCESSABLE_ENTITY));
    }
    Ok(())
}

impl FromRequest<AppState> for FormData {
    type Rejection = Refused;

    async fn from_request(request: Request, state: &AppState) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = request.into_parts();
        let req = <Req as axum::extract::FromRequestParts<AppState>>::from_request_parts(&mut parts, state)
            .await
            .map_err(|_| Refused(StatusCode::BAD_REQUEST))?;

        let bytes: Bytes = axum::body::to_bytes(body, MAX_FORM_BYTES)
            .await
            .map_err(|_| Refused(StatusCode::PAYLOAD_TOO_LARGE))?;
        let form = FormData::parse(&bytes);
        verify(&req, &parts.headers, form.get("authenticity_token"))?;
        Ok(form)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_fields_like_rails_params() {
        let form = FormData::parse(b"newsletter=0&newsletter=1&delka%5B%5D=4+m&delka%5B%5D=5+m&_method=PATCH");
        assert_eq!(form.get("newsletter"), Some("1"));
        assert_eq!(form.all("delka[]"), ["4 m", "5 m"]);
        assert_eq!(form.method(), "patch");
        assert_eq!(FormData::parse(b"").method(), "post");
    }
}
