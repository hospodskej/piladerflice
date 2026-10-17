//! The services page (Služby): one section per service, with photos.

use axum::extract::State;
use leptos::prelude::*;

use super::Result;
use super::components::format::simple_format;
use crate::AppState;
use crate::models::{content, parameterize};
use crate::request::Req;
use crate::ui::services::{ServiceSection, ServicesPage};
use crate::ui::{self};
use crate::views::Page;

pub async fn index(State(state): State<AppState>, req: Req) -> Result {
    let locale = req.locale;
    let sections: Vec<ServiceSection> = content::services_page(&state.db)
        .await?
        .iter()
        .map(|service| ServiceSection {
            anchor: anchor(
                service.button_path.as_deref().unwrap_or_default(),
                service.title.as_deref().unwrap_or_default(),
            ),
            title: service.title(locale),
            content: simple_format(&service.content(locale)),
            button_text: service
                .button_text
                .as_deref()
                .is_some_and(|text| !text.trim().is_empty())
                .then(|| service.button_text(locale)),
            images: service
                .image_list()
                .iter()
                .map(|image| state.assets.url(image))
                .collect(),
        })
        .collect();
    let page = Page::meta(&state, &req, "sluzby");
    Ok(ui::render::page(&state, &req, page, move || view! { <ServicesPage sections=sections/> }).await)
}

/// The section's id, so links like /sluzby#doprava jump to it:
/// "/sluzby#doprava" -> "doprava"; without an anchor, the title as a slug
fn anchor(button_path: &str, title: &str) -> String {
    match button_path.split('#').rfind(|part| !part.is_empty()) {
        Some(last) if !button_path.is_empty() => last.to_string(),
        _ => parameterize(title),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors() {
        assert_eq!(anchor("/sluzby#doprava", "Doprava"), "doprava");
        assert_eq!(anchor("", "Pořez / Prodej kulatiny"), "porez-prodej-kulatiny");
        assert_eq!(anchor("/sluzby", "X"), "/sluzby");
    }
}
