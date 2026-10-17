//! The kinds of firewood: soft and hard woods, with a picture each.

use leptos::prelude::*;

use crate::ui::{asset, t, t_html};

const SOFT: [&str; 3] = ["smrk", "borovice", "modrin"];
const HARD: [&str; 5] = ["buk", "habr", "dub", "akat", "jasan"];

#[component]
pub fn WoodTypes() -> impl IntoView {
    view! {
        <section class="wood-types-section">
            <h2 class="wood-types-title" inner_html=t_html("shared.wood_types.title_html")></h2>
            <div class="wood-types-layout">
                <Woods title_key="shared.wood_types.soft_title" items_key="shared.wood_types.soft_items" woods=&SOFT/>
                <Woods title_key="shared.wood_types.hard_title" items_key="shared.wood_types.hard_items" woods=&HARD/>
            </div>
        </section>
    }
}

#[component]
fn Woods(title_key: &'static str, items_key: &'static str, woods: &'static [&'static str]) -> impl IntoView {
    view! {
        <div class="wood-category-box">
            <h3>{t(title_key)}</h3>
            <div class="wood-items-grid">
                {woods
                    .iter()
                    .map(|wood| {
                        let name = t(&format!("{items_key}.{wood}"));
                        view! {
                            <div class="wood-item">
                                <img class="wood-texture" alt=name.clone() src=asset(&format!("sortiment/palivove-drevo/{wood}.webp"))/>
                                <span>{name}</span>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}
