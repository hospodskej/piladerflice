//! The quote request form (Kalkulace): contact details, then rows of items
//! for firewood or for building lumber, notes and consent.

use leptos::prelude::*;

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
            <div class="kalkulace-toggles">
                <button type="button" class="toggle-btn active" data-target="palivove">{t("kontakt.toggle_palivove")}</button>
                " "
                <button type="button" class="toggle-btn" data-target="stavebni">{t("kontakt.toggle_stavebni")}</button>
            </div>
            <form id="kalkulace-form" class="kalkulace-form" action=path("/inquiries") accept-charset="UTF-8" method="post">
                <CsrfField/>
                <input type="hidden" name="kalkulace_category" id="kalkulace-category-input" value="palivove"/>
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
    let hidden = section.category != "palivove";
    let grid = format!("kalkulace-grid-{}", section.fields.len());
    view! {
        <fieldset class="kalkulace-fields-section" data-category=section.category hidden=hidden disabled=hidden>
            <div class="kalkulace-labels-row">
                <div class=grid.clone()>
                    {section
                        .fields
                        .iter()
                        .map(|field| view! { <label class="grid-label">{t(&format!("kontakt.grid_{}", field.name))}</label> })
                        .collect_view()}
                </div>
                <div class="delete-spacer"></div>
            </div>
            <div class="kalkulace-items-container">
                <div class="kalkulace-row">
                    <div class=grid>
                        {section.fields.into_iter().map(|field| view! { <Field field=field/> }).collect_view()}
                    </div>
                    <button type="button" class="delete-item-btn" aria-label=t("kontakt.delete_item_aria")>
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm5 11H7v-2h10v2z"/>
                        </svg>
                    </button>
                </div>
            </div>
            <div class="add-item-wrapper">
                <button type="button" class="add-kalkulace-row add-item-btn" aria-label=t("kontakt.add_item_aria")>
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm5 11h-4v4h-2v-4H7v-2h4V7h2v4h4v2z"/>
                    </svg>
                </button>
            </div>
        </fieldset>
    }
}

#[component]
fn Field(field: FormField) -> impl IntoView {
    let content = if field.number {
        view! {
            <input
                type="number"
                name="mnozstvi[]"
                id="mnozstvi_"
                min="1"
                step="1"
                placeholder=t("kontakt.kalkulace_mnozstvi_placeholder")
            />
        }
        .into_any()
    } else {
        view! {
            <select name=format!("{}[]", field.name)>
                {field
                    .options
                    .into_iter()
                    .map(|option| {
                        let value = option.clone();
                        view! { <option value=value>{option}</option> }
                    })
                    .collect_view()}
                {field.custom.then(|| view! { <option value="__custom__">{t("kontakt.kalkulace_custom_option")}</option> })}
            </select>
            {field
                .custom
                .then(|| {
                    view! {
                        <input
                            type="text"
                            class="custom-value-input"
                            style="display:none"
                            placeholder=t("kontakt.kalkulace_custom_placeholder")
                        />
                    }
                })}
        }
        .into_any()
    };
    view! { <div class="form-group">{content}</div> }
}
