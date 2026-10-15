//! A refused form (forged, from another site, too big) gets the static
//! error page of `public/422.html` instead of sapwood-forms' short text.

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Response};

use crate::forms::Rejected;
use crate::views::error_pages;

pub async fn page(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    match response.extensions().get::<Rejected>() {
        Some(rejected) => (rejected.status, Html(error_pages::rejected())).into_response(),
        None => response,
    }
}
