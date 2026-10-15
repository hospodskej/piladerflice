//! Rendering a page inside the site layout (`templates/layouts/application.html`).

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use minijinja::{Value, context};

use super::page::Page;
use super::{breadcrumbs, cart_view, seo};
use crate::AppState;
use crate::locale::Locale;
use crate::request::Req;

pub const GA_MEASUREMENT_ID: &str = "G-CMJ46E7F3G";

/// Renders `template` with the layout's variables plus `data`, the page's own.
pub async fn render(state: &AppState, req: &Req, template: &str, page: Page, data: Value) -> Response {
    state.assets.refresh();
    let site_name = state.translations.text(req.locale, "site.name", &[]);

    let urls = seo::language_urls(req);
    let own_url = urls
        .iter()
        .find(|(locale, _)| *locale == req.locale)
        .map(|(_, url)| url.clone())
        .unwrap_or_default();
    let alternates: Vec<Value> = urls
        .iter()
        .map(|(locale, url)| context! { hreflang => locale.hreflang(), href => url })
        .collect();
    let x_default = urls.first().map(|(_, url)| url.clone()).unwrap_or_default();
    let og_url = if page.indexable {
        own_url.clone()
    } else {
        format!("{}{}", req.base_url, req.path)
    };

    let breadcrumbs = page
        .breadcrumbs
        .clone()
        .unwrap_or_else(|| breadcrumbs::from_path(&state.translations, req));
    let preloads: Vec<String> = state
        .importmap
        .urls(&state.assets)
        .into_iter()
        .map(|(_, url)| url)
        .collect();
    let cart = match cart_view::load(state, req).await {
        Ok(cart) => cart,
        Err(error) => {
            tracing::error!("loading the cart widget failed: {error:#}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response();
        }
    };
    let flash = page.flash_now.clone().or_else(|| req.session.take_flash());
    let current_user = req.user.as_ref().map(|current| {
        context! {
            email => &current.user.email_address,
            is_admin => current.user.is_admin(),
            avatar_path => current.avatar_path(|path| req.path_for(path)),
        }
    });

    let ctx = context! {
        locale => req.locale.code(),
        current_path => &req.plain_path,
        title => &page.title,
        description => &page.description,
        noindex => page.noindex,
        head_html => page.head_html.clone().map(Value::from_safe_string),
        indexable => page.indexable,
        canonical => own_url,
        alternates => alternates,
        x_default => x_default,
        site_name => &site_name,
        og => context! {
            url => og_url,
            locale => req.locale.og_locale(),
            alternate_locales => Locale::ALL.into_iter().filter(|l| *l != req.locale).map(Locale::og_locale).collect::<Vec<_>>(),
            image => seo::share_image(state, req),
        },
        breadcrumbs => breadcrumbs,
        switch_cs => seo::switch_path(req, Locale::Cs),
        switch_de => seo::switch_path(req, Locale::De),
        stylesheets => state.assets.urls_with_extension("css"),
        importmap_json => state.importmap.json(&state.assets),
        preloads => preloads,
        ga_measurement_id => GA_MEASUREMENT_ID,
        csrf_token => req.session.csrf_token(),
        flash => flash,
        cart => cart,
        current_user => current_user,
        ..data
    };

    match state.templates.render(template, ctx) {
        Ok(html) => (page.status.unwrap_or(StatusCode::OK), Html(html)).into_response(),
        Err(error) => {
            tracing::error!("rendering {template} failed: {error:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}
