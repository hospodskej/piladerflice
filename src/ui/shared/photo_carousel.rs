//! A few photos with arrows (Služby, the Sortiment pages); the island
//! is `islands::widgets::PhotoCarousel`.

use leptos::prelude::*;

use crate::islands::widgets;
use crate::ui::t;

/// `images` are URLs; `alt` describes them all; `class` is added to the
/// wrapper.
#[component]
pub fn PhotoCarousel(images: Vec<String>, alt: String, #[prop(optional)] class: &'static str) -> impl IntoView {
    view! {
        <widgets::PhotoCarousel
            images=images
            alt=alt
            class=class.to_string()
            previous=t("common.previous")
            next=t("common.next")
        />
    }
}
