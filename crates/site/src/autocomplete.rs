//! Suggestions while typing a town or an address (like Mapy.cz): a list
//! under the field, picked with the mouse, a tap or the keyboard (arrows,
//! Enter, Escape). A town fills the field (and can send the form at once);
//! an address fills the street, town and postcode fields.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressSuggestion {
    pub label: String,
    pub street: Option<String>,
    pub city: String,
    pub zip: Option<String>,
    pub country: String,
}

/// Suggestions for `text` in `country` ("cz"/"at"); `address` for street
/// addresses, else towns. Nothing when there's no openrouteservice key or
/// the service fails (the field still works as a plain text box).
#[server(endpoint = "naseptavac-adres", input = server_fn::codec::GetUrl)]
pub async fn suggest_addresses(text: String, country: String, address: bool) -> Result<Vec<AddressSuggestion>, ServerFnError> {
    let state = expect_context::<crate::state::AppState>();
    let ip: refractory::RemoteIp = leptos_axum::extract().await?;
    if state.limiter.check(&crate::limits::ADDRESS_SUGGESTIONS, ip.key()).await.is_err() {
        return Ok(Vec::new());
    }
    let Some(routes) = state.routes.as_ref() else { return Ok(Vec::new()) };
    let kind = if address { chemotaxis::SuggestKind::Address } else { chemotaxis::SuggestKind::Place };
    match routes.suggest(&text, &country, kind).await {
        Ok(found) => Ok(found
            .into_iter()
            .map(|s| AddressSuggestion { label: s.label, street: s.street, city: s.city, zip: s.zip, country: s.country })
            .collect()),
        Err(e) => {
            tracing::warn!("no address suggestions: {e}");
            Ok(Vec::new())
        }
    }
}

/// A text field with suggestions.
#[island]
pub fn AutocompleteInput(
    id: String,
    name: String,
    value: String,
    placeholder: String,
    /// Street addresses (fills `city_field`/`zip_field` too); else towns.
    address: bool,
    required: bool,
    /// The id of the country <select> the suggestions follow.
    #[prop(optional)]
    country_select: Option<String>,
    #[prop(optional)] city_field: Option<String>,
    #[prop(optional)] zip_field: Option<String>,
    /// Send the form when a town is picked.
    submit_on_pick: bool,
    #[prop(optional)] maxlength: Option<u32>,
) -> impl IntoView {
    let suggestions = RwSignal::new(Vec::<AddressSuggestion>::new());
    let open = RwSignal::new(false);
    let active = RwSignal::new(None::<usize>);
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let list_id = format!("{id}-suggestions");
    // Only the newest request's answer counts.
    let generation = StoredValue::new(0u64);
    let timer = StoredValue::new(None::<TimeoutHandle>);

    let close = move || {
        open.set(false);
        active.set(None);
    };
    #[allow(unused_variables)]
    let set_field = move |field_id: &Option<String>, value: &Option<String>| {
        #[cfg(feature = "hydrate")]
        if let (Some(field_id), Some(value)) = (field_id, value.as_ref().filter(|v| !v.is_empty())) {
            use wasm_bindgen::JsCast;
            let field = document().get_element_by_id(field_id);
            if let Some(input) = field.clone().and_then(|f| f.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                input.set_value(value);
            } else if let Some(select) = field.and_then(|f| f.dyn_into::<web_sys::HtmlSelectElement>().ok()) {
                select.set_value(value);
            }
        }
    };
    let country = {
        let country_select = country_select.clone();
        move || {
            #[cfg(feature = "hydrate")]
            {
                use wasm_bindgen::JsCast;
                if let Some(select) = country_select
                    .as_ref()
                    .and_then(|id| document().get_element_by_id(id))
                    .and_then(|s| s.dyn_into::<web_sys::HtmlSelectElement>().ok())
                {
                    let value = select.value();
                    if !value.is_empty() {
                        return value;
                    }
                }
            }
            let _ = &country_select;
            "cz".to_owned()
        }
    };
    let pick = {
        let (city_field, zip_field, country_select) = (city_field.clone(), zip_field.clone(), country_select.clone());
        move |index: usize| {
            let Some(s) = suggestions.get_untracked().get(index).cloned() else { return };
            close();
            if let Some(input) = input_ref.get_untracked() {
                if address {
                    if let Some(street) = &s.street {
                        input.set_value(street);
                    }
                } else {
                    input.set_value(&s.city);
                }
                set_field(&city_field, &address.then(|| s.city.clone()));
                set_field(&zip_field, &if address { s.zip.clone() } else { None });
                set_field(&country_select, &Some(s.country.clone()));
                if submit_on_pick {
                    if let Some(form) = input.form() {
                        let _ = form.request_submit();
                    }
                }
            }
        }
    };
    let on_input = {
        let country = country.clone();
        move |_| {
            if let Some(handle) = timer.get_value() {
                handle.clear();
            }
            let text = input_ref.get_untracked().map(|i| i.value()).unwrap_or_default().trim().to_owned();
            if text.chars().count() < 3 {
                close();
                return;
            }
            let country = country();
            let fetch = move || {
                let mine = generation.get_value() + 1;
                generation.set_value(mine);
                leptos::task::spawn_local(async move {
                    let found = suggest_addresses(text, country, address).await.unwrap_or_default();
                    if generation.get_value() == mine {
                        open.set(!found.is_empty());
                        active.set(None);
                        suggestions.set(found);
                    }
                });
            };
            timer.set_value(set_timeout_with_handle(fetch, std::time::Duration::from_millis(250)).ok());
        }
    };
    let on_keydown = {
        let pick = pick.clone();
        move |e: leptos::ev::KeyboardEvent| {
            if !open.get_untracked() {
                return;
            }
            let count = suggestions.with_untracked(Vec::len);
            match e.key().as_str() {
                "ArrowDown" | "ArrowUp" if count > 0 => {
                    e.prevent_default();
                    let step = if e.key() == "ArrowDown" { 1 } else { count - 1 };
                    active.set(Some(active.get_untracked().map_or(if step == 1 { 0 } else { count - 1 }, |a| (a + step) % count)));
                }
                "Enter" => {
                    if let Some(index) = active.get_untracked() {
                        e.prevent_default();
                        pick(index);
                    }
                }
                "Escape" => close(),
                _ => {}
            }
        }
    };
    let list_for_items = list_id.clone();
    let list_for_active = list_id.clone();
    view! {
        <input
            type="text"
            id=id.clone()
            name=name
            value=value
            placeholder=placeholder
            required=required
            maxlength=maxlength
            autocomplete="off"
            role="combobox"
            aria-autocomplete="list"
            aria-controls=list_id.clone()
            aria-expanded=move || open.get().to_string()
            aria-activedescendant=move || active.get().map(|a| format!("{list_for_active}-{a}"))
            node_ref=input_ref
            on:input=on_input
            on:keydown=on_keydown
            on:blur=move |_| {
                // After a tap on a suggestion (mousedown comes first).
                set_timeout(close, std::time::Duration::from_millis(150));
            }
        />
        <ul class="address-suggestions" id=list_id role="listbox" hidden=move || !open.get()>
            {move || {
                let pick = pick.clone();
                let list_id = list_for_items.clone();
                suggestions.get().into_iter().enumerate().map(move |(i, s)| {
                    let pick = pick.clone();
                    view! {
                        <li
                            id=format!("{list_id}-{i}")
                            role="option"
                            aria-selected=move || (active.get() == Some(i)).to_string()
                            on:mousedown=move |e| { e.prevent_default(); pick(i); }
                        >
                            {s.label}
                        </li>
                    }
                }).collect_view()
            }}
        </ul>
    }
}

/// The town field of "Kam vozíme?".
#[component]
pub fn PlaceInput(id: String, value: String, placeholder: String, country_select: String, #[allow(unused)] suggestions_url: String) -> impl IntoView {
    view! {
        <AutocompleteInput
            id=id
            name="query".to_owned()
            value=value
            placeholder=placeholder
            address=false
            required=true
            maxlength=80
            country_select=country_select
            submit_on_pick=true
        />
    }
}
