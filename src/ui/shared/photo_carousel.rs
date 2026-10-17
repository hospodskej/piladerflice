//! A few photos with arrows (Služby, the Sortiment pages).

use leptos::prelude::*;

use crate::ui::t;

/// `images` are URLs; `alt` describes them all; `class` is added to the
/// wrapper.
#[component]
pub fn PhotoCarousel(images: Vec<String>, alt: String, #[prop(optional)] class: &'static str) -> impl IntoView {
    view! {
        <div class=format!("sluzba-carousel-wrapper {class}").trim().to_string()>
            <button class="carousel-btn prev-btn" aria-label=t("common.previous")>
                <svg viewBox="0 0 24 24" fill="white">
                    <path d="M15.41 16.59L10.83 12l4.58-4.59L14 6l-6 6 6 6 1.41-1.41z"/>
                </svg>
            </button>
            <div class="carousel-track-container">
                <ul class="carousel-track">
                    {images
                        .into_iter()
                        .enumerate()
                        .map(|(index, src)| {
                            view! {
                                <li class="carousel-slide" class:active=index == 0>
                                    <img alt=alt.clone() src=src/>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </div>
            <button class="carousel-btn next-btn" aria-label=t("common.next")>
                <svg viewBox="0 0 24 24" fill="white">
                    <path d="M8.59 16.59L13.17 12 8.59 7.41 10 6l6 6-6 6-1.41-1.41z"/>
                </svg>
            </button>
        </div>
    }
}
