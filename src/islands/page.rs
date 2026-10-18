//! The islands of the page around the content: the theme switch, the
//! mobile menu, the logo that sticks to the top, the cookie banner, and the
//! "Ceník"/"Kontakt" menu links of the contact page.

use leptos::ev;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use super::{browser, shared};

/// "dark" or "light"; the server renders the page in that theme.
pub const THEME_COOKIE: &str = "theme";
/// The visitor's cookie choice: "all", "necessary" or "declined".
pub const CONSENT_COOKIE: &str = "cookie_consent";

const YEAR: u32 = 365 * 24 * 60 * 60;
const CONSENT_LIFETIME: u32 = 180 * 24 * 60 * 60;

/// The light/dark switch. The choice is kept in a cookie, so the next page
/// comes from the server already dark.
#[island]
pub fn ThemeToggle(id: String, label: String) -> impl IntoView {
    let toggle = move |_| {
        let dark = !browser::has_root_class("dark-mode");
        browser::set_root_class("dark-mode", dark);
        browser::set_cookie(THEME_COOKIE, if dark { "dark" } else { "light" }, YEAR);
    };
    view! {
        <button type="button" class="theme-toggle" id=id aria-label=label on:click=toggle>
            <span class="theme-toggle-track">
                <svg class="theme-icon theme-icon-sun" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="4"/>
                    <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>
                </svg>
                <svg class="theme-icon theme-icon-moon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
                </svg>
                <span class="theme-toggle-thumb"></span>
            </span>
        </button>
    }
}

/// Visitors who chose the dark theme before it was a cookie have it in
/// the browser's storage: move it over (once, when the page loads).
#[cfg(feature = "hydrate")]
pub fn adopt_stored_theme() {
    if browser::cookie(THEME_COOKIE).is_none() && browser::stored("theme").as_deref() == Some("dark") {
        browser::set_root_class("dark-mode", true);
        browser::set_cookie(THEME_COOKIE, "dark", YEAR);
    }
}

/// The ☰ button of the header (phones).
#[island]
pub fn MenuToggle(label: String) -> impl IntoView {
    view! {
        <button class="menu-toggle" id="menuToggle" aria-label=label on:click=move |_| shared::menu_open().set(true)>
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <path d="M3 12h18M3 6h18M3 18h18"/>
            </svg>
        </button>
    }
}

/// The menu that slides in on phones; `children` is its content. A click
/// beside it closes it; the page underneath doesn't scroll meanwhile.
#[island]
pub fn MobileMenu(children: Children) -> impl IntoView {
    let open = shared::menu_open();
    Effect::new(move || browser::set_body_overflow(if open.get() { "hidden" } else { "" }));
    view! {
        <div class="sidebar-overlay" id="sidebarOverlay" class:active=open on:click=move |_| open.set(false)></div>
        <aside class="mobile-sidebar" id="mobileSidebar" class:active=open>
            {children()}
        </aside>
    }
}

/// The ✕ of the mobile menu.
#[island]
pub fn MenuClose(label: String) -> impl IntoView {
    view! {
        <button class="close-sidebar" id="closeSidebar" aria-label=label on:click=move |_| shared::menu_open().set(false)>
            "✕"
        </button>
    }
}

/// The logo bar that appears once the header has scrolled away.
#[island]
pub fn StickyLogo(children: Children) -> impl IntoView {
    let visible = RwSignal::new(false);
    let update = move || visible.set(browser::scroll_y() > browser::header_height());
    Effect::new(update);
    let scroll = window_event_listener(ev::scroll, move |_| update());
    let resize = window_event_listener(ev::resize, move |_| update());
    on_cleanup(move || {
        scroll.remove();
        resize.remove();
    });
    view! {
        <div class="sticky-logo-bar" id="stickyLogoBar" class:is-visible=visible>
            {children()}
        </div>
    }
}

/// The words of the cookie banner.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CookieTexts {
    pub text: String,
    pub learn_more: String,
    pub learn_more_href: String,
    pub decline: String,
    pub necessary_only: String,
    pub accept_all: String,
}

/// Asks about cookies until the visitor answers (`show`: no answer yet),
/// and again after "Nastavení cookies" in the footer. The answer is a
/// cookie for 180 days and tells Google Analytics whether it may count.
#[island]
pub fn CookieBanner(show: bool, texts: CookieTexts) -> impl IntoView {
    let visible = shared::cookie_banner();
    if show {
        visible.set(true);
    }
    let choose = move |choice: &'static str| {
        move |_| {
            browser::set_cookie(CONSENT_COOKIE, choice, CONSENT_LIFETIME);
            browser::update_analytics_consent(choice == "all");
            visible.set(false);
        }
    };
    view! {
        <div id="cookie-consent-banner" class="cookie-consent-banner" hidden=move || !visible.get()>
            <div class="cookie-consent-content">
                <p class="cookie-consent-text">
                    {texts.text}
                    " "
                    <a class="cookie-consent-link" href=texts.learn_more_href>{texts.learn_more}</a>
                </p>
                <div class="cookie-consent-actions">
                    <button type="button" class="btn-cookie btn-cookie-decline" on:click=choose("declined")>
                        {texts.decline}
                    </button>
                    " "
                    <button type="button" class="btn-cookie btn-cookie-necessary" on:click=choose("necessary")>
                        {texts.necessary_only}
                    </button>
                    " "
                    <button type="button" class="btn-cookie btn-cookie-accept" on:click=choose("all")>
                        {texts.accept_all}
                    </button>
                </div>
            </div>
        </div>
    }
}

/// "Nastavení cookies" in the footer: shows the banner again.
#[island]
pub fn CookieSettingsButton(label: String) -> impl IntoView {
    view! {
        <button type="button" class="footer-cookie-settings" on:click=move |_| shared::cookie_banner().set(true)>
            {label}
        </button>
    }
}

/// The links of the menu that lead to the contact page: on that page
/// "Ceník" is the current one while the price list (#cenik) is shown,
/// otherwise "Kontakt".
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContactLinks {
    pub on_contact_page: bool,
    pub cenik: String,
    pub cenik_href: String,
    pub kontakt: String,
    pub kontakt_href: String,
}

#[island]
pub fn ContactNavLinks(links: ContactLinks) -> impl IntoView {
    // None on the server: it can't know the #part of the address.
    let hash = RwSignal::new(None::<String>);
    if links.on_contact_page {
        Effect::new(move || hash.set(Some(browser::location_hash())));
        let handle = window_event_listener(ev::hashchange, move |_| hash.set(Some(browser::location_hash())));
        on_cleanup(move || handle.remove());
    }
    let current = move |cenik: bool| move || hash.get().is_some_and(|hash| (hash == "#cenik") == cenik);
    view! {
        <a class="nav-link" id="nav-cenik" class:active=current(true) href=links.cenik_href>{links.cenik}</a>
        <a class="nav-link" id="nav-kontakt" class:active=current(false) href=links.kontakt_href>{links.kontakt}</a>
    }
}
