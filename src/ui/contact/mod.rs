//! The contact page: address and hours, the quote request form (Kalkulace)
//! and the price list (Ceník).

mod price_list;
mod quote_form;

pub use price_list::{PriceGroup, PriceList, PriceRow};
pub use quote_form::{FormField, FormSection};

use leptos::prelude::*;

use super::{asset, t, t_html};

#[component]
pub fn ContactPage(form_sections: Vec<FormSection>, prices: PriceList) -> impl IntoView {
    view! {
        <main class="main">
            <Details/>
            <quote_form::QuoteForm sections=form_sections/>
            <price_list::PriceListView prices=prices/>
        </main>
    }
}

/// One line of the contact details: an icon (its SVG path) and the text.
#[component]
fn Line(icon: &'static str, children: Children) -> impl IntoView {
    view! {
        <li>
            <svg viewBox="0 0 24 24" fill="currentColor" class="kontakt-icon">
                <path d=icon/>
            </svg>
            " "
            {children()}
        </li>
    }
}

const MOBILE: &str = "M17 2H7c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h10c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm-5 18c-.83 0-1.5-.67-1.5-1.5s.67-1.5 1.5-1.5 1.5.67 1.5 1.5-.67 1.5-1.5 1.5z";
const PHONE: &str = "M20 15.5c-1.25 0-2.45-.2-3.57-.57a1.02 1.02 0 0 0-1.02.24l-2.2 2.2a15.045 15.045 0 0 1-6.59-6.59l2.2-2.21a.96.96 0 0 0 .25-1A11.36 11.36 0 0 1 8.5 4c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1 0 9.39 7.61 17 17 17 .55 0 1-.45 1-1v-3.5c0-.55-.45-1-1-1z";
const MAIL: &str =
    "M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z";
const PIN: &str = "M12 2C8.13 2 5 5.13 5 9c0 5.25 7 13 7 13s7-7.75 7-13c0-3.87-3.13-7-7-7zm0 9.5c-1.38 0-2.5-1.12-2.5-2.5s1.12-2.5 2.5-2.5 2.5 1.12 2.5 2.5-1.12 2.5-2.5 2.5z";
const CLOCK: &str = "M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z";
const PERSON: &str =
    "M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z";
const BRIEFCASE: &str = "M20 6h-4V4c0-1.11-.89-2-2-2h-4c-1.11 0-2 .89-2 2v2H4c-1.11 0-1.99.89-1.99 2L2 19c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2zm-6 0h-4V4h4v2z";
const HOUSE: &str = "M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z";
const INFO: &str = "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z";

#[component]
fn Details() -> impl IntoView {
    view! {
        <section class="kontakt-page-section">
            <h1 class="kontakt-main-title">{t("kontakt.page_title")}</h1>
            <div class="kontakt-layout-grid">
                <div class="kontakt-image-wrapper">
                    <img alt=t("kontakt.image_alt") class="kontakt-image" src=asset("kontakt/drevo.webp")/>
                </div>
                <div class="kontakt-details-wrapper">
                    <div class="kontakt-titles">
                        <h2>{t("kontakt.brand_name")}</h2>
                        <h3 style="color: #2C8FFF">{t("kontakt.brand_sub")}</h3>
                    </div>
                    <div class="kontakt-info-group">
                        <h4>{t("kontakt.kontakt_title")}</h4>
                        <ul class="kontakt-list-items">
                            <Line icon=MOBILE><a href="tel:+420602446339">"+420 602 446 339"</a></Line>
                            <Line icon=PHONE><a href="tel:+420515235527">"+420 515 235 527"</a></Line>
                            <Line icon=MAIL><a href="mailto:stepan.merta@seznam.cz">"stepan.merta@seznam.cz"</a></Line>
                            <Line icon=PIN><span>{t("kontakt.address")}</span></Line>
                        </ul>
                    </div>
                    <div class="kontakt-info-group">
                        <h4>{t("kontakt.hours_title")}</h4>
                        <ul class="kontakt-list-items">
                            <Line icon=CLOCK><span inner_html=t_html("kontakt.hours_text_html")></span></Line>
                        </ul>
                    </div>
                    <div class="kontakt-info-group">
                        <h4>{t("kontakt.info_title")}</h4>
                        <ul class="kontakt-list-items">
                            <Line icon=PERSON><span>{t("kontakt.info_responsible")}</span></Line>
                            <Line icon=BRIEFCASE><span>{t("kontakt.info_ico")}</span></Line>
                            <Line icon=HOUSE><span>{t("kontakt.info_billing")}</span></Line>
                            <Line icon=INFO><span>{t("kontakt.info_legal")}</span></Line>
                        </ul>
                    </div>
                </div>
            </div>
        </section>
    }
}
