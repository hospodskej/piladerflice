//! What a handler returns when something goes wrong on our side.

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use crate::views::error_pages;

/// A database or template failure: logged, and the visitor gets the static
/// error page (public/500.html).
#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(error: E) -> Self {
        AppError(error.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("request failed: {:#}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, Html(error_pages::failed())).into_response()
    }
}

/// `Result` for handlers: a response, or an [`AppError`].
pub type Result<T = Response> = std::result::Result<T, AppError>;
