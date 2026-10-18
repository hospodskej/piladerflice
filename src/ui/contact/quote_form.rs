//! The quote request form (Kalkulace): contact details, then rows of items
//! for firewood or for building lumber, notes and consent.

use leptos::prelude::*;

use crate::islands::quote::{CategoryInput, CategoryToggles, ItemRows, QuoteField, QuoteTexts};
use crate::ui::{CsrfField, path, t, t_html, t_html_with};

/// The item rows of one category ("palivove", "stavebni").
pub struct FormSection {
    pub category: &'static str,
    pub fields: Vec<FormField>,
}

pub struct FormField {
    pub name: &'static str,
    pub options: Vec<String>,
    /// Offer "other..." with a free text field.
    pub custom: bool,
    /// Quantity of lumber: a number field instead of a list.
    pub number: bool,
}

const CONTACT_FIELDS: [(&str, &str); 8] = [
    ("jmeno", "text"),
    ("ulice", "text"),
    ("prijmeni", "text"),
    ("cislo_popisne", "text"),
    ("telefon", "tel"),
    ("mesto", "text"),
    ("email", "email"),
    ("psc", "text"),
];

#[component]
pub fn QuoteForm(sections: Vec<FormSection>) -> impl IntoView {
    let privacy_link = format!(
        r#"<a target="_blank" rel="noopener" href="{}">{}</a>"#,
        path("/zasady-ochrany-osobnich-udaju"),
        sapwood_i18n::escape_html(&t("kontakt.privacy_link_text"))
    );
    view! {
        <section class="kalkulace" id="kalkulace">
            <div class="kalkulace-header">
                <h2 inner_html=t_html("kontakt.kalkulace_title_html")></h2>
            </div>
            <CategoryToggles options=vec![
                ("palivove".to_string(), t("kontakt.toggle_palivove")),
                ("stavebni".to_string(), t("kontakt.toggle_stavebni")),
            ]/>
            <form id="kalkulace-form" class="kalkulace-form" action=path("/inquiries") accept-charset="UTF-8" method="post">
                <CsrfField/>
                <CategoryInput/>
                <h3 class="form-section-title">{t("kontakt.form_section_kontakt")}</h3>
                <div class="kalkulace-grid-2">
                    {CONTACT_FIELDS
                        .map(|(name, kind)| {
                            view! {
                                <div class="form-group">
                                    <label for=name>{t(&format!("kontakt.form_{name}"))}</label>
                                    <input required="required" type=kind name=name id=name/>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
                <h3 class="form-section-title">{t("kontakt.form_section_formular")}</h3>
                {sections.into_iter().map(|section| view! { <Items section=section/> }).collect_view()}
                <h3 class="form-section-title no-border">{t("kontakt.form_section_poznamky")}</h3>
                <div class="form-group">
                    <textarea rows="6" name="poznamky" id="poznamky"></textarea>
                </div>
                <div class="form-consent">
                    <label class="custom-checkbox-label">
                        <input required="required" type="checkbox" value="1" name="souhlas" id="souhlas"/>
                        <span class="custom-checkbox"></span>
                        " "
                        <span class="consent-text" inner_html=t_html_raw_link("kontakt.consent_text_html", &privacy_link)></span>
                    </label>
                </div>
                <div class="form-submit-wrapper">
                    <input type="submit" value=t("kontakt.submit_button") class="btn-primary"/>
                </div>
            </form>
        </section>
    }
}

/// `key`'s HTML with `{privacy_link}` replaced by a link built here (HTML
/// already, so not escaped again).
fn t_html_raw_link(key: &str, link: &str) -> String {
    t_html_with(key, &[("privacy_link", "\u{0}LINK\u{0}")]).replace("\u{0}LINK\u{0}", link)
}

/// The item rows of one category; only firewood shows at first.
#[component]
fn Items(section: FormSection) -> impl IntoView {
    let texts = QuoteTexts {
        labels: section
            .fields
            .iter()
            .map(|field| t(&format!("kontakt.grid_{}", field.name)))
            .collect(),
        delete: t("kontakt.delete_item_aria"),
        add: t("kontakt.add_item_aria"),
        custom_option: t("kontakt.kalkulace_custom_option"),
        custom_placeholder: t("kontakt.kalkulace_custom_placeholder"),
        quantity_placeholder: t("kontakt.kalkulace_mnozstvi_placeholder"),
    };
    // The wide rows' lists open as the site's own menus.
    let dropdowns = matches!(section.fields.len(), 4 | 5);
    let fields = section
        .fields
        .into_iter()
        .map(|field| QuoteField {
            name: field.name.to_string(),
            options: field.options,
            custom: field.custom,
            number: field.number,
        })
        .collect();
    view! { <ItemRows category_key=section.category.to_string() fields=fields dropdowns=dropdowns texts=texts/> }
}
