//! Turbo Streams: small pieces of HTML Turbo puts into the current page
//! without reloading it ("replace the cart widget", "show the modal").

use axum::http::header;
use axum::response::{IntoResponse, Response};

/// One `<turbo-stream>`: `action` ("replace", "update") the element with id
/// `target` by `html`.
pub fn stream(action: &str, target: &str, html: &str) -> String {
    format!(r#"<turbo-stream action="{action}" target="{target}"><template>{html}</template></turbo-stream>"#)
}

pub fn response(streams: Vec<String>) -> Response {
    (
        [(header::CONTENT_TYPE, "text/vnd.turbo-stream.html; charset=utf-8")],
        streams.join("\n"),
    )
        .into_response()
}
