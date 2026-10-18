//! POST /api/...: the islands' server functions (`islands/`). They get the
//! site's state as context; a function reads the visitor's request (the
//! session, the signed-in user) itself.
//!
//! Forms carry the form token; these calls are protected by their origin
//! instead: browsers send `Origin` with every such request, and only the
//! site's own pages can name this site there. (The session cookie is also
//! SameSite=Lax, so another site's request wouldn't carry it.)

use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use leptos::prelude::provide_context;

use crate::AppState;

pub async fn handle(State(state): State<AppState>, request: Request) -> Response {
    if !same_origin(&request) {
        return (StatusCode::FORBIDDEN, "cross-site request").into_response();
    }
    leptos_axum::handle_server_fns_with_context(move || provide_context(state.clone()), request)
        .await
        .into_response()
}

/// Whether the request comes from a page of this site: its Origin names the
/// host it was sent to.
fn same_origin(request: &Request) -> bool {
    let header = |name| request.headers().get(name).and_then(|value| value.to_str().ok());
    let (Some(origin), Some(host)) = (header(header::ORIGIN), header(header::HOST)) else {
        return false;
    };
    origin
        .split_once("://")
        .is_some_and(|(_, origin_host)| origin_host.eq_ignore_ascii_case(host))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(origin: Option<&str>) -> Request {
        let mut builder = Request::builder()
            .uri("/api/cart/add")
            .header("host", "piladerflice.cz");
        if let Some(origin) = origin {
            builder = builder.header("origin", origin);
        }
        builder.body(Default::default()).unwrap()
    }

    #[test]
    fn only_the_sites_own_pages_may_call() {
        assert!(same_origin(&request(Some("https://piladerflice.cz"))));
        assert!(!same_origin(&request(Some("https://evil.example"))));
        assert!(!same_origin(&request(Some("https://piladerflice.cz.evil.example"))));
        assert!(!same_origin(&request(None)));
    }
}
