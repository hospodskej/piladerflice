//! The legal pages: one long HTML translation each.

use leptos::prelude::*;

use super::{t, t_html};

#[component]
pub fn LegalPage(title_key: &'static str, content_key: &'static str) -> impl IntoView {
    view! {
        <main class="main">
            <section class="legal-page">
                <h1>{t(title_key)}</h1>
                <div class="legal-content" inner_html=t_html(content_key)></div>
            </section>
        </main>
    }
}
