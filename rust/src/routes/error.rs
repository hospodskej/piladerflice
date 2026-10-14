//! What a handler returns when something goes wrong on our side.

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

/// A database or template failure: logged, and the visitor gets the static
/// error page (Rails' public/500.html).
pub struct AppError(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(error: E) -> Self {
        AppError(error.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("request failed: {:#}", self.0);
        let page = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../public/500.html"))
            .unwrap_or_else(|_| "Internal Server Error".into());
        (StatusCode::INTERNAL_SERVER_ERROR, Html(page)).into_response()
    }
}

/// `Result` for handlers: a response, or an [`AppError`].
pub type Result<T = Response> = std::result::Result<T, AppError>;
