//! The admin's pages (/admin): Czech only, never indexed. The handlers in
//! `routes/admin/` load the records into the structs here; the components
//! only arrange them, inside [`AdminLayout`] (the menu on the left, the
//! signed-in admin, the message of the last form).
//!
//! - [`form`]: the fields of a record's form, the buttons that post
//! - one module per section, like `routes/admin/`

pub mod dashboard;
pub mod form;
pub mod inquiries;
pub mod options;
pub mod orders;
pub mod pricelist;
pub mod products;
pub mod promos;
pub mod variants;

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use leptos::prelude::*;

use super::Ui;
use super::layout::IslandScripts;
use super::render::document;
use crate::AppState;
use crate::request::Req;
use crate::session::Flash;
use form::ActionButton;

/// What the layout shows around every admin page.
pub struct Chrome {
    /// "/admin/orders/5": which menu item is current.
    current_path: String,
    admin_email: Option<String>,
    flash: Option<Flash>,
    stylesheets: Vec<String>,
    pkg_path: Option<String>,
}

/// An admin page: `content` inside the layout, with `status` (422 for a
/// form that didn't save).
pub fn page<F, V>(state: &AppState, req: &Req, status: StatusCode, content: F) -> Response
where
    F: FnOnce() -> V + Send + 'static,
    V: IntoView + 'static,
{
    state.assets.refresh();
    let chrome = Chrome {
        current_path: req.path.clone(),
        admin_email: req.user.as_ref().map(|current| current.user.email_address.clone()),
        flash: req.session.take_flash(),
        stylesheets: state.assets.urls_with_extension("css"),
        pkg_path: state.wasm.pkg_path(),
    };
    let html = document(Ui::new(state, req), move || {
        view! { <AdminLayout chrome=chrome>{content()}</AdminLayout> }
    });
    (status, Html(html)).into_response()
}

const SECTIONS: [(&str, &str); 7] = [
    ("Přehled", "/admin"),
    ("Katalog a ceny", "/admin/catalog_products"),
    ("Ceník", "/admin/pricelist_items"),
    ("Doporučujeme", "/admin/promos"),
    ("Objednávky", "/admin/orders"),
    ("Poptávky", "/admin/inquiries"),
    ("Formulář poptávky", "/admin/inquiry_form_options"),
];

#[component]
fn AdminLayout(chrome: Chrome, children: Children) -> impl IntoView {
    let Chrome {
        current_path,
        admin_email,
        flash,
        stylesheets,
        pkg_path,
    } = chrome;
    let flash = flash.map(|flash| {
        let (kind, text) = match flash {
            Flash::Notice(text) => ("notice", text),
            Flash::Alert(text) => ("alert", text),
        };
        view! { <div class=format!("admin-flash admin-flash-{kind}")>{text}</div> }
    });
    view! {
        <html lang="cs">
            <head>
                <title>"Administrace | Pila Derflice"</title>
                <meta name="viewport" content="width=device-width,initial-scale=1"/>
                <meta name="robots" content="noindex, nofollow"/>
                {stylesheets.into_iter().map(|href| view! { <link rel="stylesheet" href=href/> }).collect_view()}
                <IslandScripts pkg_path=pkg_path/>
            </head>
            <body class="admin-body">
                <div class="admin-layout">
                    <aside class="admin-sidebar">
                        <div class="admin-sidebar-brand">"Pila Derflice"<span>"Administrace"</span></div>
                        <nav class="admin-nav">
                            {SECTIONS
                                .map(|(label, url)| {
                                    let active = if url == "/admin" {
                                        current_path == url
                                    } else {
                                        current_path.starts_with(url)
                                    };
                                    view! { <a class="admin-nav-link" class:active=active href=url>{label}</a> }
                                })
                                .collect_view()}
                        </nav>
                        <div class="admin-sidebar-footer">
                            <span class="admin-current-user">{admin_email}</span>
                            <a class="admin-back-link" href="/">"← Zpět na web"</a>
                            <ActionButton label="Odhlásit se" action="/odhlaseni".into() class="admin-logout-btn"/>
                        </div>
                    </aside>
                    <main class="admin-main">{flash} {children()}</main>
                </div>
            </body>
        </html>
    }
}

/// "Ano" or "Ne".
fn yes_no(yes: bool) -> &'static str {
    if yes { "Ano" } else { "Ne" }
}
