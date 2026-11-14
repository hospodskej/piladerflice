//! The page for an address that doesn't exist.

use super::PageMeta;
use crate::i18n::t;
use crate::layout::Layout;
use crate::visitor::visitor;
use leptos::prelude::*;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    #[cfg(feature = "ssr")]
    if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
        response.set_status(axum::http::StatusCode::NOT_FOUND);
    }
    let v = visitor();
    let l = v.locale;
    use t::not_found as n;
    view! {
        <PageMeta title=n::title(l).to_owned()/>
        <leptos_meta::Meta name="robots" content="noindex"/>
        <Layout>
            <main class="main">
                <section class="not-found">
                    // The shrugging mascot in front of a black and white photo.
                    <div class="not-found-hero">
                        <img src="/img/not-found-photo.webp" alt="" class="not-found-photo" width="1600" height="667" loading="eager"/>
                        <img src="/img/mascot/shrugging.webp" alt=n::alt(l) class="not-found-mascot" width="640" height="475" loading="eager"/>
                    </div>
                    <h1>{n::heading(l)}</h1>
                    <p class="not-found-text">{n::text(l)}</p>
                    <a href=v.link("/") class="btn-primary">{n::home(l)}</a>
                    <nav class="not-found-links">
                        <a href=v.link("/eshop")>{n::eshop(l)}</a>
                        <a href=v.link("/sortiment")>{n::sortiment(l)}</a>
                        <a href=v.link("/kontakt")>{n::kontakt(l)}</a>
                    </nav>
                </section>
            </main>
        </Layout>
    }
}
