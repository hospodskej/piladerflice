//! "How to order": online, or by asking for a quote.

use leptos::prelude::*;

use crate::ui::{asset, path, t, t_html, t_list};

#[component]
pub fn HowToOrder() -> impl IntoView {
    view! {
        <h2 class="section-main-title jak-objednat-title" inner_html=t_html("shared.jak_objednat.title_html")></h2>
        <div class="jak-objednat-grid">
            <Way kind="online" image="jak-objednat/drevo.webp" link="/eshop"/>
            <Way kind="poptavka" image="jak-objednat/pila.webp" link="/kontakt#kalkulace"/>
        </div>
    }
}

/// One way of ordering: its steps, a picture and where to start.
#[component]
fn Way(kind: &'static str, image: &'static str, link: &'static str) -> impl IntoView {
    let key = |name: &str| format!("shared.jak_objednat.{kind}_{name}");
    view! {
        <div class="jak-objednat-box">
            <h3>
                {t(&key("title_rest"))}
                " "
                <span class="text-blue">{t(&key("title_accent"))}</span>
            </h3>
            <ul class="jak-objednat-steps">
                {t_list(&key("steps"))
                    .into_iter()
                    .enumerate()
                    .map(|(index, step)| view! { <li><strong>{format!("{}.", index + 1)}</strong>" "{step}</li> })
                    .collect_view()}
            </ul>
            <img class="jak-objednat-box-img" alt=t(&key("img_alt")) src=asset(image)/>
            <a class="btn-primary full-width" href=path(link)>{t(&key("button"))}</a>
        </div>
    }
}
