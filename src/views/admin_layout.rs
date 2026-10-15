//! Rendering an admin page inside `templates/layouts/admin.html`: the
//! sidebar menu, the signed-in admin's e-mail, the messages. The admin is
//! Czech only and never indexed.

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use minijinja::{Value, context};

use crate::AppState;
use crate::request::Req;

pub fn render(state: &AppState, req: &Req, template: &str, data: Value) -> Response {
    render_with_status(state, req, StatusCode::OK, template, data)
}

/// For a form that didn't save: 422, so Turbo shows it in place.
pub fn render_with_status(state: &AppState, req: &Req, status: StatusCode, template: &str, data: Value) -> Response {
    state.assets.refresh();
    let preloads: Vec<String> = state
        .importmap
        .urls(&state.assets)
        .into_iter()
        .map(|(_, url)| url)
        .collect();
    let ctx = context! {
        locale => "cs",
        current_path => &req.path,
        stylesheets => state.assets.urls_with_extension("css"),
        importmap_json => state.importmap.json(&state.assets),
        preloads => preloads,
        csrf_token => req.session.csrf_token(),
        flash => req.session.take_flash(),
        admin_email => req.user.as_ref().map(|current| current.user.email_address.clone()),
        ..data
    };
    match state.templates.render(template, ctx) {
        Ok(html) => (status, Html(html)).into_response(),
        Err(error) => {
            tracing::error!("rendering {template} failed: {error:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}
