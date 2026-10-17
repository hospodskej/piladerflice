//! The services accordion, the "might interest you" slides and the
//! frequent questions.

use leptos::prelude::*;

use super::Accented;
use crate::ui::{asset, path, t};

/// A service in the accordion; a button only when it has text and a link.
pub struct ServiceItem {
    pub title: String,
    /// Trusted HTML typed in the admin (it has <br> line breaks).
    pub content: String,
    pub button: Option<(String, String)>,
}

/// A "might interest you" slide.
pub struct Slide {
    pub title: String,
    /// Trusted HTML typed in the admin.
    pub content: String,
    /// A logical asset name.
    pub image: Option<String>,
}

#[component]
pub fn Services(services: Vec<ServiceItem>) -> impl IntoView {
    view! {
        <section class="sluzby-section">
            <h2><Accented rest="home.index.sluzby_title_rest" accent="home.index.sluzby_title_accent"/></h2>
            <div class="accordion">
                {services
                    .into_iter()
                    .map(|service| {
                        view! {
                            <div class="accordion-item">
                                <button class="accordion-header">
                                    <span class="accordion-title">{service.title}</span>
                                    " "
                                    <span class="accordion-icon">
                                        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                                            <polyline points="6 9 12 15 18 9"/>
                                        </svg>
                                    </span>
                                </button>
                                <div class="accordion-content-wrapper">
                                    <div class="accordion-content">
                                        <div class="accordion-body">
                                            <p inner_html=service.content></p>
                                            {service
                                                .button
                                                .map(|(text, href)| view! { <a class="btn-cenik" href=href>{text}</a> })}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

#[component]
pub fn Slides(slides: Vec<Slide>) -> impl IntoView {
    let count = slides.len();
    view! {
        <section class="co-by-vas-mohlo-zajimat">
            <h2><Accented rest="home.index.zajimat_title_rest" accent="home.index.zajimat_title_accent"/></h2>
            <div class="carousel-wrapper" id="zajimatCarousel">
                <button class="carousel-btn prev-btn" id="carouselPrev" aria-label=t("common.previous")>
                    <ChevronLeft/>
                </button>
                <div class="carousel-container">
                    {slides
                        .into_iter()
                        .enumerate()
                        .map(|(index, slide)| {
                            view! {
                                <div class="carousel-slide" class:active=index == 0>
                                    <img class="carousel-bg-img" alt=slide.title.clone() src=slide.image.as_deref().map(asset)/>
                                    <div class="carousel-overlay"></div>
                                    <div class="carousel-content">
                                        <h3>{slide.title}</h3>
                                        <div class="carousel-text">
                                            <p inner_html=slide.content></p>
                                        </div>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
                <button class="carousel-btn next-btn" id="carouselNext" aria-label=t("common.next")>
                    <ChevronRight/>
                </button>
            </div>
            <div class="carousel-dots">
                {(1..=count)
                    .map(|number| {
                        view! {
                            <button
                                class="carousel-dot"
                                class:active=number == 1
                                aria-label=format!("{} {number}", t("common.slide"))
                            ></button>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

/// The carousels' arrows.
#[component]
pub fn ChevronLeft() -> impl IntoView {
    view! {
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 18 9 12 15 6"/>
        </svg>
    }
}

#[component]
pub fn ChevronRight() -> impl IntoView {
    view! {
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="9 18 15 12 9 6"/>
        </svg>
    }
}

/// Four questions in each of three topics.
#[component]
pub fn Faq() -> impl IntoView {
    view! {
        <section class="faq-section">
            <h2 class="faq-main-title"><Accented rest="home.index.faq_title_rest" accent="home.index.faq_title_accent"/></h2>
            {["palivove", "stavebni", "doprava"]
                .map(|topic| {
                    view! {
                        <h3 class="faq-category-title">{t(&format!("home.index.faq_cat_{topic}"))}</h3>
                        <div class="faq-grid">
                            {(1..5)
                                .map(|number| {
                                    view! {
                                        <div class="accordion-item faq-item">
                                            <button class="accordion-header">
                                                <span class="accordion-title">{t(&format!("home.index.faq_{topic}_q{number}"))}</span>
                                                " "
                                                <span class="faq-icon"></span>
                                            </button>
                                            <div class="accordion-content-wrapper">
                                                <div class="accordion-content">
                                                    <div class="accordion-body">
                                                        <p>{t(&format!("home.index.faq_{topic}_a{number}"))}</p>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                })
                .collect_view()}
            <div class="faq-cta">
                <div class="faq-cta-left">
                    <span class="faq-cta-icon">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M20 15.5c-1.25 0-2.45-.2-3.57-.57a1.02 1.02 0 0 0-1.02.24l-2.2 2.2a15.045 15.045 0 0 1-6.59-6.59l2.2-2.21a.96.96 0 0 0 .25-1A11.36 11.36 0 0 1 8.5 4c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1 0 9.39 7.61 17 17 17 .55 0 1-.45 1-1v-3.5c0-.55-.45-1-1-1z"/>
                        </svg>
                    </span>
                    " "
                    <span>{t("home.index.faq_cta_text")}</span>
                </div>
                <a class="btn-primary" href=path("/kontakt")>{t("home.index.faq_cta_button")}</a>
            </div>
        </section>
    }
}
