//! The record ids in admin URLs (`/admin/orders/{id}`). An id that isn't a
//! number gets the "page not found" page, like an unknown record does.

use std::collections::HashMap;

use axum::extract::{FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::response::Response;

use crate::AppState;
use crate::request::Req;
use crate::routes::not_found;

/// The `{id}` of the URL.
pub struct RecordId(pub i64);

/// The `{product_id}` of a variant's URL.
pub struct ProductId(pub i64);

async fn param(parts: &mut Parts, state: &AppState, name: &str) -> Result<i64, Response> {
    let params = Path::<HashMap<String, String>>::from_request_parts(parts, state)
        .await
        .ok();
    match params.and_then(|params| params.get(name).and_then(|id| id.parse().ok())) {
        Some(id) => Ok(id),
        None => Err(not_found_page(parts, state).await),
    }
}

async fn not_found_page(parts: &mut Parts, state: &AppState) -> Response {
    match Req::from_request_parts(parts, state).await {
        Ok(req) => not_found::page(State(state.clone()), req).await,
        Err(error) => axum::response::IntoResponse::into_response(error),
    }
}

impl FromRequestParts<AppState> for RecordId {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        param(parts, state, "id").await.map(RecordId)
    }
}

impl FromRequestParts<AppState> for ProductId {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        param(parts, state, "product_id").await.map(ProductId)
    }
}
