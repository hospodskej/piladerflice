//! The administration (Czech only): the catalogue and prices, the price
//! list, delivery, VAT, discount codes, promos, orders and inquiries.

pub mod catalog;
pub mod content;
pub mod form;
pub mod orders;
pub mod settings;

use crate::visitor::{Flash, Visitor, visitor};
use leptos::prelude::*;

/// The shop's money in admin, always in crowns: "1 740 Kč".
pub fn czk(amount: i64) -> String {
    domain::Czk(amount).to_string()
}

/// "13.11.2026 21:05" in Prague time, from a stored UTC timestamp.
pub fn when(iso: &str) -> String {
    #[cfg(feature = "ssr")]
    {
        use time::format_description::well_known::Iso8601;
        if let Ok(at) = time::OffsetDateTime::parse(iso, &Iso8601::DEFAULT) {
            return crate::time_fmt::format_cz(at);
        }
    }
    iso.to_owned()
}

/// An admin page: the guard (admins only; never cached or indexed) and the
/// frame with the menu.
#[component]
pub fn AdminLayout(children: Children) -> impl IntoView {
    let v = visitor();
    let allowed = v.user.as_ref().is_some_and(|u| u.admin);
    // Others never get here (see `handlers::admin_only`).
    if !allowed {
        return ().into_any();
    }
    let at = |prefix: &str| {
        if v.path.starts_with(prefix) {
            "admin-nav-link active"
        } else {
            "admin-nav-link"
        }
    };
    let link = |href: &'static str, prefix: &'static str, text: &'static str| view! { <a href=href class=at(prefix)>{text}</a> };
    view! {
        <leptos_meta::Title text="Administrace | Pila Derflice"/>
        <leptos_meta::Meta name="robots" content="noindex, nofollow"/>
        <leptos_meta::Body {..} class="admin-body"/>
        <div class="admin-layout">
            <aside class="admin-sidebar">
                <div class="admin-sidebar-brand">"Pila Derflice"<span>"Administrace"</span></div>
                <nav class="admin-nav">
                    <a href="/admin" class=if v.path == "/admin" { "admin-nav-link active" } else { "admin-nav-link" }>"Přehled"</a>
                    {link("/admin/catalog_products", "/admin/catalog_products", "Katalog a ceny")}
                    {link("/admin/pricelist_items", "/admin/pricelist_items", "Ceník")}
                    {link("/admin/doprava/edit", "/admin/doprava", "Doprava")}
                    {link("/admin/dph", "/admin/dph", "DPH")}
                    {link("/admin/slevove-kody", "/admin/slevove-kody", "Slevové kódy")}
                    {link("/admin/promos", "/admin/promos", "Doporučujeme")}
                    {link("/admin/orders", "/admin/orders", "Objednávky")}
                    {link("/admin/inquiries", "/admin/inquiries", "Poptávky")}
                    {link("/admin/inquiry_form_options", "/admin/inquiry_form_options", "Formulář poptávky")}
                </nav>
                <div class="admin-sidebar-footer">
                    <span class="admin-current-user">{v.user.as_ref().map(|u| u.email.clone())}</span>
                    <a href="/" class="admin-site-link">"← Zpět na web"</a>
                    <form class="button_to" method="post" action="/odhlaseni">
                        <button class="admin-logout-btn" type="submit">"Odhlásit se"</button>
                    </form>
                </div>
            </aside>
            <main class="admin-main">
                {v.flash.clone().map(|f| match f {
                    Flash::Notice(text) => view! { <div class="admin-flash admin-flash-notice">{text}</div> },
                    Flash::Alert(text) => view! { <div class="admin-flash admin-flash-alert">{text}</div> },
                })}
                {children()}
            </main>
        </div>
        <crate::islands::AdminBehaviour/>
    }
    .into_any()
}

/// A file input for a photo: the chosen photo is shrunk in the browser
/// (at most 1600 px, WebP) before it's uploaded, so the server needs no
/// image library.
#[island]
pub fn ShrinkingFileInput(id: String, name: String) -> impl IntoView {
    view! {
        <input
            type="file"
            id=id
            name=name
            accept="image/*"
            on:change=move |e| {
                #[cfg(feature = "hydrate")]
                {
                    use wasm_bindgen::JsCast;
                    if let Some(input) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                        leptos::task::spawn_local(async move {
                            // On an error the original is uploaded; the server checks it anyway.
                            let _ = attenuator::Shrink::new().shrink_input(&input).await;
                        });
                    }
                }
                let _ = e;
            }
        />
    }
}

/// A delete (or other) button that asks first.
#[component]
pub fn ConfirmButton(
    action: String,
    text: &'static str,
    #[prop(into)] confirm: String,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    view! {
        <form class="button_to" method="post" action=action data-confirm=confirm>
            <button class=class.unwrap_or("admin-danger-btn") type="submit">{text}</button>
        </form>
    }
}

/// For server functions and handlers: the signed-in admin, or an error.
pub fn is_admin(v: &Visitor) -> bool {
    v.user.as_ref().is_some_and(|u| u.admin)
}

#[cfg(feature = "ssr")]
pub fn require_admin() -> Result<Visitor, ServerFnError> {
    let v = visitor();
    if is_admin(&v) {
        Ok(v)
    } else {
        Err(ServerFnError::new("admins only"))
    }
}

#[cfg(feature = "ssr")]
pub mod handlers {
    //! What the admin forms post to.
    pub use super::catalog::handlers as catalog;
    pub use super::content::handlers as content;
    pub use super::orders::handlers as orders;
    pub use super::settings::handlers as settings;

    use crate::visitor::{Flash, Visitor};
    use axum::response::{IntoResponse, Redirect, Response};
    use tower_sessions::Session;

    /// Middleware for everything under /admin: admins only (others go to
    /// the home page with a message); never cached or indexed.
    pub async fn admin_only(
        axum::Extension(v): axum::Extension<Visitor>,
        session: Session,
        request: axum::extract::Request,
        next: axum::middleware::Next,
    ) -> Response {
        let path = request.uri().path();
        if path != "/admin" && !path.starts_with("/admin/") {
            return next.run(request).await;
        }
        if let Err(refused) = guard(&session, &v).await {
            return refused;
        }
        let mut response = next.run(request).await;
        let headers = response.headers_mut();
        headers.insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store"),
        );
        headers.insert(
            axum::http::HeaderName::from_static("x-robots-tag"),
            axum::http::HeaderValue::from_static("noindex, nofollow"),
        );
        response
    }

    /// Admins only; others go to the home page with a message.
    pub async fn guard(session: &Session, v: &Visitor) -> Result<(), Response> {
        if super::is_admin(v) {
            Ok(())
        } else {
            let _ = crate::session::flash(
                session,
                Flash::Alert("Nemáte oprávnění k přístupu do administrace.".into()),
            )
            .await;
            Err(Redirect::to("/").into_response())
        }
    }

    /// Leaves `notice` and redirects (303) to an admin page.
    pub async fn done(session: &Session, path: &str, notice: impl Into<String>) -> Response {
        let _ = crate::session::flash(session, Flash::Notice(notice.into())).await;
        Redirect::to(path).into_response()
    }

    pub const REFUSED: &str = "admin_refused";

    /// Back to the form with what was typed and why it can't be saved.
    pub async fn refused(
        session: &Session,
        path: &str,
        values: super::form::Values,
        errors: Vec<String>,
    ) -> Response {
        let _ = session
            .insert(
                REFUSED,
                super::form::Refused {
                    path: path.to_owned(),
                    values,
                    errors,
                },
            )
            .await;
        Redirect::to(path).into_response()
    }

    /// Saves an uploaded photo (checked and stripped of metadata by
    /// spliceosome) under a name made from its content; returns its address.
    pub async fn save_upload(
        state: &crate::state::AppState,
        bytes: &[u8],
    ) -> Result<String, String> {
        use sha2::Digest;
        let limits = spliceosome::Limits {
            max_bytes: 15 * 1024 * 1024,
            max_pixels: 50_000_000,
        };
        let clean = spliceosome::sanitize(bytes, &limits).map_err(|e| match e {
            spliceosome::Error::TooLarge { .. } => "Obrázek je příliš velký.".to_owned(),
            spliceosome::Error::UnsupportedFormat => {
                "Obrázek musí být JPG, PNG nebo WebP.".to_owned()
            }
            _ => "Obrázek se nepodařilo přečíst.".to_owned(),
        })?;
        let digest = sha2::Sha256::digest(clean.bytes());
        let name: String = digest.iter().take(12).map(|b| format!("{b:02x}")).collect();
        let file = format!("{name}.{}", clean.extension());
        let dir = state.storage.join("uploads");
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| e.to_string())?;
        tokio::fs::write(dir.join(&file), clean.bytes())
            .await
            .map_err(|e| e.to_string())?;
        Ok(format!("/uploads/{file}"))
    }

    pub async fn failed(session: &Session, path: &str, alert: impl Into<String>) -> Response {
        let _ = crate::session::flash(session, Flash::Alert(alert.into())).await;
        Redirect::to(path).into_response()
    }
}
