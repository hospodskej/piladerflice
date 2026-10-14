//! The friendly 404 page, with one of the three mascots picked at random.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use minijinja::context;
use rand::seq::IndexedRandom;

use crate::AppState;
use crate::request::Req;
use crate::views::{self, Page};

const MASCOTS: [&str; 3] = [
    "mascot/waving.webp",
    "mascot/thumbs-up.webp",
    "mascot/hands-on-hips.webp",
];

/// Any unknown URL. /at/whatever gets the German page. A missing file
/// (/logo.png, /feed.xml) gets a bare 404, not a whole page.
pub async fn page(State(state): State<AppState>, req: Req) -> Response {
    let last = req.plain_path.rsplit('/').next().unwrap_or_default();
    if last.rsplit_once('.').is_some_and(|(_, ext)| ext != "html") {
        return StatusCode::NOT_FOUND.into_response();
    }

    let title = state.translations.text(req.locale, "not_found.title", &[]);
    let mut page = Page::titled(&state, &req, &title, None);
    page.noindex = true;
    page.breadcrumbs = Some(Vec::new());
    page.status = Some(StatusCode::NOT_FOUND);

    let mascot = *MASCOTS.choose(&mut rand::rng()).unwrap_or(&MASCOTS[0]);
    views::render(&state, &req, "errors/not_found.html", page, context! { mascot })
}
