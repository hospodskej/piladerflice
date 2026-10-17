//! The services page (Služby): one section per service, with photos.

use leptos::prelude::*;

use super::path;
use super::shared::PhotoCarousel;

pub struct ServiceSection {
    /// The section's id, for links like /sluzby#doprava.
    pub anchor: String,
    pub title: String,
    /// HTML made from the text typed in the admin.
    pub content: String,
    pub button_text: Option<String>,
    /// Photo URLs.
    pub images: Vec<String>,
}

#[component]
pub fn ServicesPage(sections: Vec<ServiceSection>) -> impl IntoView {
    view! {
        <main class="main">
            <section class="sluzby-page-section">
                {sections
                    .into_iter()
                    .enumerate()
                    .map(|(index, section)| {
                        view! {
                            <div class="sluzba-row" class:reverse-layout=index % 2 == 1 id=section.anchor>
                                <div class="sluzba-content">
                                    <h2>{section.title.clone()}</h2>
                                    <div class="sluzba-text" inner_html=section.content></div>
                                    {section
                                        .button_text
                                        .map(|text| view! { <a class="btn-primary" href=path("/kontakt#cenik-sluzeb")>{text}</a> })}
                                </div>
                                {(!section.images.is_empty())
                                    .then(|| view! { <PhotoCarousel images=section.images alt=section.title/> })}
                            </div>
                        }
                    })
                    .collect_view()}
            </section>
        </main>
    }
}
