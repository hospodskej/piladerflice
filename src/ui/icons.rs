//! Small SVG pictures used in several places.

use leptos::prelude::*;

/// The green phone of the contact lines (header, mobile menu).
#[component]
pub fn PhoneIcon(size: u32) -> impl IntoView {
    view! {
        <svg width=size height=size fill="none" stroke="#30b109" stroke-width="2" viewBox="0 0 24 24">
            <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"/>
        </svg>
    }
}

/// The green envelope of the contact lines.
#[component]
pub fn MailIcon(size: u32) -> impl IntoView {
    view! {
        <svg width=size height=size fill="none" stroke="#30b109" stroke-width="2" viewBox="0 0 24 24">
            <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/>
            <polyline points="22,6 12,13 2,6"/>
        </svg>
    }
}

/// The shopping cart.
#[component]
pub fn CartIcon(#[prop(optional)] aria_hidden: bool) -> impl IntoView {
    view! {
        <svg
            viewBox="0 0 24 24"
            width="22"
            height="22"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden=aria_hidden.then_some("true")
        >
            <circle cx="9" cy="21" r="1"/>
            <circle cx="20" cy="21" r="1"/>
            <path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"/>
        </svg>
    }
}

/// The Czech flag (language switcher).
#[component]
pub fn FlagCz() -> impl IntoView {
    view! {
        <svg width="20" height="15" viewBox="0 0 600 400" xmlns="http://www.w3.org/2000/svg" class="flag-icon" style="border-radius: 2px">
            <rect width="600" height="400" fill="#d7141a" class="flag-red"/>
            <rect width="600" height="200" fill="#fff"/>
            <polygon points="0,0 0,400 300,200" fill="#11457e"/>
        </svg>
    }
}

/// The Austrian flag (language switcher).
#[component]
pub fn FlagAt() -> impl IntoView {
    view! {
        <svg width="20" height="15" viewBox="0 0 600 400" xmlns="http://www.w3.org/2000/svg" class="flag-icon" style="border-radius: 2px">
            <rect width="600" height="400" fill="#ed2939" class="flag-red"/>
            <rect width="600" height="133.33" y="133.33" fill="#fff"/>
        </svg>
    }
}
