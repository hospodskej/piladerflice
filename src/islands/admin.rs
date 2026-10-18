//! The admin's island: forms that ask before they post ("Opravdu smazat?").

use leptos::ev::SubmitEvent;
use leptos::prelude::*;

/// A form that posts to `action` once the admin confirms `question`;
/// `children` is its content (the form token, the button).
#[island]
pub fn ConfirmForm(action: String, question: String, children: Children) -> impl IntoView {
    let ask = move |event: SubmitEvent| {
        if !window().confirm_with_message(&question).unwrap_or(false) {
            event.prevent_default();
        }
    };
    view! {
        <form class="action-button" method="post" action=action on:submit=ask>
            {children()}
        </form>
    }
}
