//! The quote request form's islands (Kalkulace on the contact page): the
//! category switch (firewood or building lumber), the rows of items, and
//! the item lists, which offer "other..." with a free text field.
//!
//! Before the WebAssembly is there the form still works with the
//! browser's own lists, one row per category.

use std::sync::atomic::{AtomicU64, Ordering};

use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// The category shown: "palivove" or "stavebni".
fn category() -> RwSignal<String> {
    super::shared::quote_category()
}

/// The two buttons that switch the category.
#[island]
pub fn CategoryToggles(options: Vec<(String, String)>) -> impl IntoView {
    let current = category();
    view! {
        <div class="kalkulace-toggles">
            {options
                .into_iter()
                .enumerate()
                .map(|(index, (value, label))| {
                    let this = value.clone();
                    view! {
                        {(index > 0).then_some(" ")}
                        <button
                            type="button"
                            class="toggle-btn"
                            class:active=move || current.with(|current| *current == this)
                            on:click=move |_| current.set(value.clone())
                        >
                            {label}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// The form field that sends the category.
#[island]
pub fn CategoryInput() -> impl IntoView {
    let current = category();
    view! { <input type="hidden" name="kalkulace_category" id="kalkulace-category-input" value=move || current.get()/> }
}

/// A column of the item rows.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuoteField {
    /// The form field is `{name}[]`.
    pub name: String,
    pub options: Vec<String>,
    /// Offer "other..." with a free text field.
    pub custom: bool,
    /// A number field instead of a list (quantity of lumber).
    pub number: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuoteTexts {
    /// The column headings.
    pub labels: Vec<String>,
    pub delete: String,
    pub add: String,
    /// "Jiné..."
    pub custom_option: String,
    pub custom_placeholder: String,
    pub quantity_placeholder: String,
}

/// The value of the "other..." choice.
const CUSTOM: &str = "__custom__";

/// The item rows of one category, with + to add a row and − to remove one.
/// Hidden (and not sent) while the other category is shown. `dropdowns`:
/// the lists open as the site's own menus.
#[island]
pub fn ItemRows(category_key: String, fields: Vec<QuoteField>, dropdowns: bool, texts: QuoteTexts) -> impl IntoView {
    let current = category();
    let category_key = StoredValue::new(category_key);
    let hidden = move || current.with(|current| category_key.with_value(|key| current != key));
    let grid = format!("kalkulace-grid-{}", fields.len());
    let next_id = StoredValue::new(1_u32);
    let rows = RwSignal::new(vec![0_u32]);
    let add = move |_| {
        let id = next_id.get_value();
        next_id.set_value(id + 1);
        rows.update(|rows| rows.push(id));
    };
    let fields = StoredValue::new(fields);
    let texts = StoredValue::new(texts);
    let row_grid = grid.clone();
    let row = move |id: u32| {
        let remove = move |_| rows.update(|rows| rows.retain(|row| *row != id));
        let texts = texts.get_value();
        view! {
            <div class="kalkulace-row">
                <div class=row_grid.clone()>
                    {fields
                        .get_value()
                        .into_iter()
                        .map(|field| view! { <Field field=field dropdown=dropdowns texts=texts.clone()/> })
                        .collect_view()}
                </div>
                <button type="button" class="delete-item-btn" aria-label=texts.delete.clone() on:click=remove>
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm5 11H7v-2h10v2z"/>
                    </svg>
                </button>
            </div>
        }
    };
    view! {
        <fieldset class="kalkulace-fields-section" hidden=hidden disabled=hidden>
            <div class="kalkulace-labels-row">
                <div class=grid>
                    {texts
                        .get_value()
                        .labels
                        .into_iter()
                        .map(|label| view! { <label class="grid-label">{label}</label> })
                        .collect_view()}
                </div>
                <div class="delete-spacer"></div>
            </div>
            <div class="kalkulace-items-container">
                <For each=move || rows.get() key=|id| *id children=row/>
            </div>
            <div class="add-item-wrapper">
                <button type="button" class="add-kalkulace-row add-item-btn" aria-label=texts.get_value().add on:click=add>
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm5 11h-4v4h-2v-4H7v-2h4V7h2v4h4v2z"/>
                    </svg>
                </button>
            </div>
        </fieldset>
    }
}

#[component]
fn Field(field: QuoteField, dropdown: bool, texts: QuoteTexts) -> impl IntoView {
    let content = if field.number {
        view! {
            <input
                type="number"
                name="mnozstvi[]"
                id="mnozstvi_"
                min="1"
                step="1"
                placeholder=texts.quantity_placeholder
            />
        }
        .into_any()
    } else {
        view! { <Choice field=field dropdown=dropdown texts=texts/> }.into_any()
    };
    view! { <div class="form-group">{content}</div> }
}

/// Which list's menu is open; only one at a time, and a click anywhere
/// else closes it.
fn open_menu() -> RwSignal<Option<u64>> {
    super::shared::open_menu()
}

/// A list of options, with "other..." when `field.custom`. The browser's
/// list is the form field; with `dropdown`, once the island is alive, it
/// hides behind the site's own menu. "Other..." disables the list and sends
/// the text field instead.
#[component]
fn Choice(field: QuoteField, dropdown: bool, texts: QuoteTexts) -> impl IntoView {
    static NEXT_MENU: AtomicU64 = AtomicU64::new(0);
    let menu_id = NEXT_MENU.fetch_add(1, Ordering::Relaxed);
    let name = format!("{}[]", field.name);
    let mut labels = field.options.clone();
    let mut values = field.options.clone();
    if field.custom {
        labels.push(texts.custom_option.clone());
        values.push(CUSTOM.to_string());
    }
    let custom_index = field.custom.then(|| labels.len() - 1);
    // Every preset option has a unit ("33 cm"): then "other..." takes a
    // number and adds the unit.
    let numeric =
        field.custom && !field.options.is_empty() && field.options.iter().all(|option| !unit(option).is_empty());
    let labels = StoredValue::new(labels);

    let enhanced = RwSignal::new(false);
    if dropdown {
        Effect::new(move || enhanced.set(true));
    }
    let selected = RwSignal::new(0_usize);
    // The option chosen before "other...", to go back to.
    let previous = RwSignal::new(0_usize);
    let custom_value = RwSignal::new(String::new());
    let editing = RwSignal::new(false);
    let custom_chosen = move || Some(selected.get()) == custom_index;
    let menus = open_menu();
    let open = move || menus.get() == Some(menu_id);
    let input = NodeRef::<html::Input>::new();
    let select = NodeRef::<html::Select>::new();

    let choose = move |index: usize| {
        if Some(index) == custom_index && !custom_chosen() {
            previous.set(selected.get_untracked());
        }
        selected.set(index);
        menus.set(None);
        if Some(index) == custom_index {
            if numeric {
                custom_value.update(|value| *value = leading_number(value));
            }
            editing.set(true);
            request_animation_frame(move || {
                if let Some(input) = input.get_untracked() {
                    let _ = input.focus();
                }
            });
        } else {
            editing.set(false);
        }
    };
    let finish = move || {
        let value = custom_value.get_untracked().trim().to_string();
        if value.is_empty() {
            choose(previous.get_untracked());
            return;
        }
        let unit = labels.with_value(|labels| unit(&labels[previous.get_untracked()]).to_string());
        let value = if is_number(&value) && !unit.is_empty() {
            format!("{value} {unit}")
        } else {
            value
        };
        custom_value.set(value);
        editing.set(false);
    };
    let shown = move || {
        if custom_chosen() && !editing.get() && !custom_value.with(String::is_empty) {
            custom_value.get()
        } else {
            labels.with_value(|labels| labels[selected.get()].clone())
        }
    };
    let close = window_event_listener(ev::click, move |_| {
        if menus.get_untracked() == Some(menu_id) {
            menus.set(None);
        }
    });
    on_cleanup(move || close.remove());
    // The browser's list follows the choice (it's what the form sends).
    Effect::new(move || {
        let index = selected.get();
        if let Some(select) = select.get() {
            select.set_selected_index(index as i32);
        }
    });

    view! {
        <div class="custom-select-wrapper">
            <select
                name=name.clone()
                node_ref=select
                disabled=custom_chosen
                style=move || enhanced.get().then_some("display: none")
                on:change=move |event| {
                    let index = event_target::<web_sys::HtmlSelectElement>(&event).selected_index();
                    choose(usize::try_from(index).unwrap_or(0));
                }
            >
                {values
                    .into_iter()
                    .zip(labels.get_value())
                    .map(|(value, label)| view! { <option value=value>{label}</option> })
                    .collect_view()}
            </select>
            {move || {
                enhanced
                    .get()
                    .then(|| {
                        view! {
                            <div
                                class="custom-select-display"
                                class:open=open
                                on:click=move |event: ev::MouseEvent| {
                                    event.stop_propagation();
                                    menus.set(if open() { None } else { Some(menu_id) });
                                }
                            >
                                {move || Formatted(shown())}
                            </div>
                            <div class="custom-select-menu" class:open=open>
                                {labels
                                    .get_value()
                                    .into_iter()
                                    .enumerate()
                                    .map(|(index, label)| {
                                        view! {
                                            <div
                                                class="custom-select-item"
                                                on:click=move |event: ev::MouseEvent| {
                                                    event.stop_propagation();
                                                    choose(index);
                                                }
                                            >
                                                {Formatted(label)}
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        }
                    })
            }}
        </div>
        {field
            .custom
            .then(|| {
                view! {
                    <input
                        type="text"
                        class="custom-value-input"
                        node_ref=input
                        style=move || if editing.get() { "display: block" } else { "display:none" }
                        name=move || custom_chosen().then(|| name.clone())
                        required=custom_chosen
                        inputmode=numeric.then_some("decimal")
                        placeholder=texts.custom_placeholder
                        prop:value=custom_value
                        on:input=move |event| {
                            let mut value = event_target_value(&event);
                            if numeric {
                                value.retain(|c| c.is_ascii_digit() || c == '.' || c == ',');
                            }
                            custom_value.set(value);
                        }
                        on:keydown=move |event: ev::KeyboardEvent| {
                            if event.key() == "Enter" {
                                event.prevent_default();
                                if let Some(input) = input.get_untracked() {
                                    let _ = input.blur();
                                }
                            }
                        }
                        on:blur=move |_| finish()
                    />
                }
            })}
    }
}

/// An option as the menu shows it: the first word blue ("33 cm").
#[allow(non_snake_case)]
fn Formatted(text: String) -> impl IntoView {
    let text = text.trim().to_string();
    match text.split_once(' ') {
        Some((first, rest)) => view! {
            <span style="color: #2C8FFF">{first.to_string()}</span>
            " "
            <span>{rest.to_string()}</span>
        }
        .into_any(),
        None => view! { <span style="color: #2C8FFF">{text}</span> }.into_any(),
    }
}

/// "33 cm" → "cm"; "" when the text isn't a number with a unit.
fn unit(text: &str) -> &str {
    let text = text.trim();
    let digits = text.find(|c: char| !c.is_ascii_digit()).unwrap_or(text.len());
    if digits == 0 {
        return "";
    }
    let mut rest = &text[digits..];
    if let Some(fraction) = rest.strip_prefix(['.', ',']) {
        let more = fraction.find(|c: char| !c.is_ascii_digit()).unwrap_or(fraction.len());
        if more > 0 {
            rest = &fraction[more..];
        }
    }
    rest.trim()
}

/// "12", "1,5": a number, nothing else.
fn is_number(text: &str) -> bool {
    !text.is_empty() && unit(text).is_empty() && text.starts_with(|c: char| c.is_ascii_digit()) && {
        let tail = text.trim_start_matches(|c: char| c.is_ascii_digit());
        tail.is_empty()
            || (tail.len() > 1 && tail[1..].chars().all(|c| c.is_ascii_digit()) && tail.starts_with(['.', ',']))
    }
}

/// The number a text starts with ("12 m3" → "12"), or "".
fn leading_number(text: &str) -> String {
    let text = text.trim_start();
    let whole = text.find(|c: char| !c.is_ascii_digit()).unwrap_or(text.len());
    if whole == 0 {
        return String::new();
    }
    let rest = &text[whole..];
    let fraction = rest
        .strip_prefix(['.', ','])
        .map(|fraction| fraction.find(|c: char| !c.is_ascii_digit()).unwrap_or(fraction.len()))
        .filter(|digits| *digits > 0)
        .map_or(0, |digits| digits + 1);
    text[..whole + fraction].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_and_numbers() {
        assert_eq!(unit("33 cm"), "cm");
        assert_eq!(unit("1,5 m3"), "m3");
        assert_eq!(unit("Buk"), "");
        assert_eq!(unit("12"), "");
        assert!(is_number("12") && is_number("1,5") && is_number("2.25"));
        assert!(!is_number("12 m") && !is_number("") && !is_number("1,") && !is_number("x"));
        assert_eq!(leading_number("12 m3"), "12");
        assert_eq!(leading_number("1,5 m"), "1,5");
        assert_eq!(leading_number("1, m"), "1");
        assert_eq!(leading_number("abc"), "");
    }
}
