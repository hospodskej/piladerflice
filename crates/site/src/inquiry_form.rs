//! The inquiry form on the contact page ("Kalkulace"): firewood and/or
//! lumber rows, each a set of dropdowns. Rendered with the page on the
//! server; in the browser rows can be added and removed and the dropdowns
//! open, take a custom value, and so on.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// The value of the "other value" option.
pub const CUSTOM: &str = "__custom__";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormField {
    /// "druh", "delka"...
    pub key: String,
    pub label: String,
    pub options: Vec<String>,
    /// Offers "Jiná hodnota" with a text box.
    pub custom: bool,
    /// A plain number box instead of a dropdown (lumber quantity).
    pub number: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormCategory {
    /// "palivove" or "stavebni".
    pub key: String,
    pub title: String,
    pub fields: Vec<FormField>,
}

/// Every text the form shows, already in the visitor's language.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormTexts {
    pub action: String,
    pub toggle_hint: String,
    pub section_formular: String,
    pub section_poznamky: String,
    pub custom_option: String,
    pub custom_placeholder: String,
    pub quantity_placeholder: String,
    pub delete_item: String,
    pub add_item: String,
    /// Trusted HTML (with the privacy policy link).
    pub consent_html: String,
    pub submit: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InquiryFormData {
    pub categories: Vec<FormCategory>,
    pub texts: FormTexts,
}

/// "5 PRM" → the number in blue, then the rest. Text, never HTML: it comes
/// from the admin or the customer.
#[component]
fn OptionText(#[prop(into)] text: Signal<String>) -> impl IntoView {
    move || {
        let text = text.get();
        let mut parts = text.trim().splitn(2, ' ');
        let number = parts.next().unwrap_or_default().to_owned();
        let rest = parts.next().map(str::to_owned);
        view! {
            <span class="custom-select-number">{number}</span>
            {rest.map(|r| view! { " "<span>{r}</span> })}
        }
    }
}

/// The unit after a leading number: "5 PRM" → "PRM", "Jiná" → "".
fn unit_of(text: &str) -> &str {
    let text = text.trim();
    let digits = text
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ','))
        .unwrap_or(text.len());
    if digits == 0 || !text.starts_with(|c: char| c.is_ascii_digit()) {
        return "";
    }
    text[digits..].trim()
}

fn is_number(text: &str) -> bool {
    let text = text.trim();
    !text.is_empty()
        && text.starts_with(|c: char| c.is_ascii_digit())
        && unit_of(text).is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
}

/// A dropdown styled like the rest of the form. The chosen value travels in
/// a hidden input named like the old <select>.
#[component]
fn CustomSelect(
    name: String,
    options: Vec<String>,
    custom: bool,
    texts: FormTexts,
) -> impl IntoView {
    let selected = RwSignal::new(0usize);
    let restore = RwSignal::new(0usize);
    let custom_value = RwSignal::new(String::new());
    let editing = RwSignal::new(false);
    let open = RwSignal::new(false);
    let custom_index = options.len();
    // A field whose options are all "number unit" takes a number as its custom value.
    let numeric = custom && !options.is_empty() && options.iter().all(|o| !unit_of(o).is_empty());
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let labels = {
        let mut labels = options.clone();
        if custom {
            labels.push(texts.custom_option.clone());
        }
        labels
    };
    let label_of = {
        let labels = labels.clone();
        move |i: usize| labels.get(i).cloned().unwrap_or_default()
    };
    let display_text = {
        let label_of = label_of.clone();
        Signal::derive(move || {
            if selected.get() == custom_index && !editing.get() && !custom_value.get().is_empty() {
                custom_value.get()
            } else {
                label_of(selected.get())
            }
        })
    };
    let value = {
        let options = options.clone();
        move || {
            if selected.get() == custom_index {
                custom_value.get()
            } else {
                options.get(selected.get()).cloned().unwrap_or_default()
            }
        }
    };
    let options_for_finish = options.clone();
    let finish_custom = move || {
        let mut text = custom_value.get_untracked().trim().to_owned();
        if text.is_empty() {
            selected.set(restore.get_untracked());
            editing.set(false);
            return;
        }
        if is_number(&text) {
            let unit = unit_of(
                options_for_finish
                    .get(restore.get_untracked())
                    .map_or("", String::as_str),
            )
            .to_owned();
            if !unit.is_empty() {
                text = format!("{text} {unit}");
            }
        }
        custom_value.set(text);
        editing.set(false);
    };

    #[cfg(feature = "hydrate")]
    let _ = window_event_listener(leptos::ev::click, move |_| open.set(false));

    let choose = move |i: usize| {
        if i == custom_index && selected.get_untracked() != custom_index {
            restore.set(selected.get_untracked());
        }
        selected.set(i);
        open.set(false);
        if i == custom_index {
            if numeric {
                let current = custom_value.get_untracked();
                let number: String = current
                    .trim()
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
                    .collect();
                custom_value.set(number);
            }
            editing.set(true);
            #[cfg(feature = "hydrate")]
            request_animation_frame(move || {
                if let Some(input) = input_ref.get_untracked() {
                    let _ = input.focus();
                }
            });
        } else {
            editing.set(false);
        }
    };

    view! {
        <div class="custom-select-wrapper">
            <input type="hidden" name=name prop:value=value/>
            <div
                class="custom-select-display"
                class:open=open
                on:click=move |e| {
                    e.stop_propagation();
                    open.update(|o| *o = !*o);
                }
            >
                <OptionText text=display_text/>
            </div>
            <div class="custom-select-menu" class:open=open>
                {labels.iter().enumerate().map(|(i, label)| {
                    let label = label.clone();
                    view! {
                        <div class="custom-select-item" on:click=move |e| { e.stop_propagation(); choose(i); }>
                            <OptionText text=label/>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
        {custom.then(|| view! {
            <input
                type="text"
                class="custom-value-input"
                style:display=move || if editing.get() { "block" } else { "none" }
                placeholder=texts.custom_placeholder.clone()
                inputmode=numeric.then_some("decimal")
                required=move || editing.get()
                node_ref=input_ref
                prop:value=custom_value
                on:input=move |e| {
                    let mut text = event_target_value(&e);
                    if numeric {
                        text.retain(|c| c.is_ascii_digit() || c == '.' || c == ',');
                    }
                    custom_value.set(text);
                }
                on:keydown=move |e| if e.key() == "Enter" {
                    e.prevent_default();
                    if let Some(input) = input_ref.get_untracked() { let _ = input.blur(); }
                }
                on:blur=move |_| finish_custom()
            />
        })}
    }
}

#[component]
fn ItemRow(category: FormCategory, texts: FormTexts, on_delete: Callback<()>) -> impl IntoView {
    let columns = category.fields.len();
    view! {
        <div class="kalkulace-row">
            <div class=format!("kalkulace-grid-{columns}")>
                {category.fields.iter().map(|field| {
                    let name = format!("items[{}][{}][]", category.key, field.key);
                    view! {
                        <div class="form-group">
                            // On phones the boxes stack, so each carries its own name.
                            <span class="kalkulace-field-label" aria-hidden="true">{field.label.clone()}</span>
                            {if field.number {
                                view! { <input type="number" name=name min="1" step="1" placeholder=texts.quantity_placeholder.clone()/> }.into_any()
                            } else {
                                view! { <CustomSelect name=name options=field.options.clone() custom=field.custom texts=texts.clone()/> }.into_any()
                            }}
                        </div>
                    }
                }).collect_view()}
            </div>
            <button type="button" class="delete-item-btn" aria-label=texts.delete_item.clone() on:click=move |_| on_delete.run(())>
                <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm5 11H7v-2h10v2z"/></svg>
            </button>
        </div>
    }
}

#[island]
pub fn InquiryForm(data: InquiryFormData, children: Children) -> impl IntoView {
    let texts = data.texts.clone();
    // Switches, not tabs: firewood, lumber or both in one inquiry; one stays on.
    let active: RwSignal<Vec<bool>> = RwSignal::new(
        data.categories
            .iter()
            .enumerate()
            .map(|(i, _)| i == 0)
            .collect(),
    );
    let toggle = move |i: usize| {
        active.update(|a| {
            let turning_off = a[i];
            if turning_off && a.iter().filter(|x| **x).count() == 1 {
                return;
            }
            a[i] = !turning_off;
        });
    };

    view! {
        <div class="kalkulace-toggles">
            {data.categories.iter().enumerate().map(|(i, c)| view! {
                <button
                    type="button"
                    class="toggle-btn"
                    class:active=move || active.get()[i]
                    data-target=c.key.clone()
                    aria-pressed=move || active.get()[i].to_string()
                    on:click=move |_| toggle(i)
                >
                    {c.title.clone()}
                </button>
            }).collect_view()}
        </div>
        <p class="kalkulace-toggles-hint">{texts.toggle_hint.clone()}</p>
        <form action=texts.action.clone() method="post" class="kalkulace-form" id="kalkulace-form">
            {children()}
            <h3 class="form-section-title">{texts.section_formular.clone()}</h3>
            {data.categories.iter().enumerate().map(|(i, category)| {
                let next_id = StoredValue::new(1usize);
                let rows = RwSignal::new(vec![0usize]);
                let off = move || !active.get()[i];
                let category = category.clone();
                let texts = texts.clone();
                let (row_category, row_texts) = (category.clone(), texts.clone());
                let columns = category.fields.len();
                view! {
                    <fieldset class="kalkulace-fields-section" data-category=category.key.clone() prop:hidden=off prop:disabled=off>
                        <legend class="kalkulace-category-title">{category.title.clone()}</legend>
                        <div class="kalkulace-labels-row">
                            <div class=format!("kalkulace-grid-{columns}")>
                                {category.fields.iter().map(|f| view! { <label class="grid-label">{f.label.clone()}</label> }).collect_view()}
                            </div>
                            <div class="delete-spacer"></div>
                        </div>
                        <div class="kalkulace-items-container">
                            <For each=move || rows.get() key=|id| *id let:id>
                                <ItemRow
                                    category=row_category.clone()
                                    texts=row_texts.clone()
                                    on_delete=Callback::new(move |()| rows.update(|r| r.retain(|x| *x != id)))
                                />
                            </For>
                        </div>
                        <div class="add-item-wrapper">
                            <button
                                type="button"
                                class="add-kalkulace-row add-item-btn"
                                aria-label=texts.add_item.clone()
                                on:click=move |_| {
                                    let id = next_id.get_value();
                                    next_id.set_value(id + 1);
                                    rows.update(|r| r.push(id));
                                }
                            >
                                <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm5 11h-4v4h-2v-4H7v-2h4V7h2v4h4v2z"/></svg>
                            </button>
                        </div>
                    </fieldset>
                }
            }).collect_view()}
            <h3 class="form-section-title no-border">{texts.section_poznamky.clone()}</h3>
            <div class="form-group">
                <textarea name="poznamky" id="poznamky" rows="6"></textarea>
            </div>
            <div class="form-consent">
                <label class="custom-checkbox-label">
                    <input type="checkbox" name="souhlas" id="souhlas" value="1" required/>
                    <span class="custom-checkbox"></span>
                    <span class="consent-text" inner_html=texts.consent_html.clone()></span>
                </label>
            </div>
            <div class="form-submit-wrapper">
                <input type="submit" value=texts.submit.clone() class="btn-primary"/>
            </div>
        </form>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units() {
        assert_eq!(unit_of("5 PRM"), "PRM");
        assert_eq!(unit_of("2,5 m"), "m");
        assert_eq!(unit_of("Jiná hodnota"), "");
        assert!(is_number("12"));
        assert!(is_number("2,5"));
        assert!(!is_number("12 m"));
    }
}
