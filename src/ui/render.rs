//! Turning components into HTML responses. A page is rendered in one go on
//! the server (no streaming): the handler loads everything first, then the
//! components only arrange it.

use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use hydration_context::SsrSharedContext;
use leptos::prelude::*;
use leptos::tachys::view::RenderHtml;

use super::Ui;
use super::layout::{Layout, LayoutData};
use crate::AppState;
use crate::request::Req;
use crate::views::Page;
use crate::views::cart_view::CartView;

/// The session's cart, for pages that show it besides the header (the cart
/// page, the checkout).
pub fn current_cart() -> CartView {
    expect_context::<CartView>()
}

/// Renders `build` with `ui` as context. Islands inside get the markers the
/// browser needs to bring them to life.
pub fn html<V: RenderHtml>(ui: Ui, build: impl FnOnce() -> V) -> String {
    let owner = Owner::new_root(Some(Arc::new(SsrSharedContext::new_islands())));
    owner.with(|| {
        provide_context(ui);
        build().to_html()
    })
}

/// A whole document: `<!DOCTYPE html>` and `build`'s `<html>` element.
pub fn document<V: RenderHtml>(ui: Ui, build: impl FnOnce() -> V) -> String {
    format!("<!DOCTYPE html>{}", html(ui, build))
}

/// A page of the site: `content` inside the layout (head, header,
/// breadcrumbs, footer), with the page's status. The session's cart is in
/// the context too ([`current_cart`]), as the header shows it.
pub async fn page<F, V>(state: &AppState, req: &Req, page: Page, content: F) -> Response
where
    F: FnOnce() -> V + Send + 'static,
    V: IntoView + 'static,
{
    let status = page.status.unwrap_or(StatusCode::OK);
    let data = match LayoutData::load(state, req, page).await {
        Ok(data) => data,
        Err(error) => {
            tracing::error!("loading the page layout failed: {error:#}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(crate::views::error_pages::failed()),
            )
                .into_response();
        }
    };
    let html = document(Ui::new(state, req), move || {
        provide_context(data.cart.clone());
        view! { <Layout data=data>{content()}</Layout> }
    });
    (status, Html(html)).into_response()
}
