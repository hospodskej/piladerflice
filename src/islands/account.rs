//! The account page's islands: the phone prefix picker and the profile
//! picture's file picker.

use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::icons::{FlagAt, FlagCz};

/// +420 or +43, with the flag; sent as `phone_prefix`.
#[island]
pub fn PhonePrefix(prefix: String, label: String) -> impl IntoView {
    let prefix = RwSignal::new(prefix);
    // Where the menu opens (under the button, which may sit in a scrolled
    // box); None while closed.
    let menu_at = RwSignal::new(None::<String>);
    let root = NodeRef::<html::Div>::new();
    let button = NodeRef::<html::Button>::new();
    let toggle = move |event: ev::MouseEvent| {
        event.stop_propagation();
        if menu_at.get_untracked().is_some() {
            menu_at.set(None);
            return;
        }
        let Some(button) = button.get_untracked() else {
            return;
        };
        let rect = button.get_bounding_client_rect();
        menu_at.set(Some(format!(
            "position: fixed; top: {}px; left: {}px; min-width: {}px",
            rect.bottom() + 4.0,
            rect.left(),
            rect.width()
        )));
    };
    let outside = window_event_listener(ev::click, move |event| {
        let inside = root.get_untracked().zip(event.target()).is_some_and(|(root, target)| {
            use wasm_bindgen::JsCast;
            target
                .dyn_ref::<web_sys::Node>()
                .is_some_and(|node| root.contains(Some(node)))
        });
        if !inside {
            menu_at.set(None);
        }
    });
    on_cleanup(move || outside.remove());
    let flag = move |value: &str| {
        if value == "+43" {
            view! { <FlagAt/> }.into_any()
        } else {
            view! { <FlagCz/> }.into_any()
        }
    };
    let option = move |value: &'static str| {
        view! {
            <div
                class="phone-prefix-option"
                on:click=move |event: ev::MouseEvent| {
                    event.stop_propagation();
                    prefix.set(value.to_string());
                    menu_at.set(None);
                }
            >
                {flag(value)}
                <span>{value}</span>
            </div>
        }
    };
    let open = move || menu_at.with(Option::is_some);
    view! {
        <div class="phone-prefix-dropdown" node_ref=root>
            <button type="button" class="phone-prefix-btn" class:open=open node_ref=button aria-label=label on:click=toggle>
                <span>{move || flag(&prefix.get())}</span>
                " "
                <span>{prefix}</span>
                " "
                <span class="chevron">"▼"</span>
            </button>
            <input type="hidden" name="phone_prefix" value=prefix/>
            <div class="phone-prefix-menu" class:open=open style=move || menu_at.get()>
                {option("+420")}
                {option("+43")}
            </div>
        </div>
    }
}

/// The words and the address of the profile picture form.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AvatarTexts {
    pub action: String,
    pub choose: String,
    /// "No file chosen"
    pub empty: String,
    pub upload: String,
}

/// Choosing and uploading a profile picture: a button in the site's style
/// instead of the browser's file field (which stays in the form, hidden
/// but focusable), the chosen file's name, and the upload button, usable
/// once a file is chosen. `children`: the form token.
#[island]
pub fn AvatarUpload(texts: AvatarTexts, children: Children) -> impl IntoView {
    let file = RwSignal::new(None::<String>);
    // The upload button waits for a file only once the island is alive.
    let alive = RwSignal::new(false);
    Effect::new(move || alive.set(true));
    let chosen = move |event: ev::Event| {
        let input = event_target::<web_sys::HtmlInputElement>(&event);
        file.set(input.files().and_then(|files| files.get(0)).map(|file| file.name()));
    };
    let empty = texts.empty.clone();
    view! {
        <form
            class="avatar-upload-form"
            enctype="multipart/form-data"
            action=texts.action
            accept-charset="UTF-8"
            method="post"
        >
            {children()}
            <input
                type="file"
                name="avatar"
                id="avatar_file"
                class="file-picker-input"
                accept="image/jpeg,image/png,image/webp"
                required=true
                on:change=chosen
            />
            <label for="avatar_file" class="btn-outline-dark file-picker-btn">{texts.choose}</label>
            " "
            <span class="file-picker-name" class:has-file=move || file.with(Option::is_some)>
                {move || file.get().unwrap_or_else(|| empty.clone())}
            </span>
            " "
            <input
                type="submit"
                value=texts.upload
                class="btn-primary avatar-btn"
                disabled=move || alive.get() && file.with(Option::is_none)
            />
        </form>
    }
}
