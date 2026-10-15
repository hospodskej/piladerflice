//! /muj-ucet/profilovy-obrazek: the customer's picture. Shown only to its
//! owner (from here, behind the login), uploaded through
//! [`sanitize`](crate::accounts::avatar::sanitize), removed on request.

use std::sync::LazyLock;
use std::time::Duration;

use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use super::require_login;
use crate::AppState;
use crate::accounts::avatar::{self, sanitize};
use crate::accounts::rate_limit::RateLimit;
use crate::forms::{FormData, UploadForm};
use crate::request::Req;
use crate::routes::Result;
use crate::session::Flash;
use crate::views::redirect;

/// Processing pictures is work; at most 10 uploads per 10 minutes each.
static UPLOADS: LazyLock<RateLimit> = LazyLock::new(|| RateLimit::new(10, Duration::from_secs(600)));

pub async fn show(State(state): State<AppState>, req: Req) -> Result {
    let current = match require_login(&state, &req) {
        Ok(current) => current,
        Err(redirect) => return Ok(*redirect),
    };
    let bytes = match &current.avatar {
        Some(found) => avatar::read(&state.config.storage_root, found).await,
        None => None,
    };
    let Some(bytes) = bytes else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let mut response = Response::new(Body::from(bytes));
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/webp"));
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("inline; filename=\"avatar.webp\""),
    );
    // The URL carries ?v=<checksum>, so a changed picture is a new URL.
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("max-age=31536000, private"),
    );
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; sandbox"),
    );
    Ok(response)
}

/// POST /muj-ucet/profilovy-obrazek: a new picture (multipart form).
pub async fn update(State(state): State<AppState>, req: Req, form: UploadForm) -> Result {
    let current = match require_login(&state, &req) {
        Ok(current) => current,
        Err(redirect) => return Ok(*redirect),
    };
    let back = req.path_for("/muj-ucet/upravit");
    let alert = |reason: &str| {
        req.session.set_flash(Flash::Alert(state.translations.text(
            req.locale,
            &format!("auth.avatar_errors.{reason}"),
            &[],
        )));
        Ok(redirect::to(&back))
    };
    if !UPLOADS.allow(current.user.id) {
        return alert("rate_limited");
    }
    let Some(file) = form.file("avatar") else {
        return alert("none_selected");
    };
    // Decoding and encoding is CPU work: off the async threads.
    let bytes = file.bytes.clone();
    let webp = tokio::task::spawn_blocking(move || sanitize::sanitize(&bytes)).await?;
    match webp {
        Ok(webp) => {
            avatar::store(&state.db, &state.config.storage_root, current.user.id, &webp).await?;
            req.session.set_flash(Flash::Notice(state.translations.text(
                req.locale,
                "auth.avatar_updated",
                &[],
            )));
            Ok(redirect::to(&back))
        }
        Err(invalid) => alert(invalid.reason()),
    }
}

/// POST /muj-ucet/profilovy-obrazek/smazat: removes the picture.
pub async fn destroy(State(state): State<AppState>, req: Req, _form: FormData) -> Result {
    let current = match require_login(&state, &req) {
        Ok(current) => current,
        Err(redirect) => return Ok(*redirect),
    };
    avatar::remove(&state.db, &state.config.storage_root, current.user.id).await?;
    req.session.set_flash(Flash::Notice(state.translations.text(
        req.locale,
        "auth.avatar_removed",
        &[],
    )));
    Ok(redirect::to(&req.path_for("/muj-ucet/upravit")))
}
