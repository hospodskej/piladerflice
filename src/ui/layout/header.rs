//! The header: contact line, menu, account, cart and language.

use leptos::prelude::*;

use super::data::UserBadge;
use crate::islands::cart::{CartView, CartWidget, WidgetTexts};
use crate::islands::page::{self, ContactLinks, ContactNavLinks, MenuToggle};
use crate::locale::Locale;
use crate::ui::icons::{FlagAt, FlagCz, MailIcon, PhoneIcon};
use crate::ui::{asset, current_path, locale, path, t};

#[component]
pub fn Header(user: Option<UserBadge>, cart: CartView, switch_cs: String, switch_de: String) -> impl IntoView {
    view! {
        <header class="site-header">
            <div class="contact-header">
                <div class="contact-item">
                    <PhoneIcon size=14/>
                    " "
                    <a href="tel:+420602446339" class="contact-link">"+420 602 446 339"</a>
                </div>
                <div class="contact-item">
                    <MailIcon size=14/>
                    " "
                    <a href="mailto:stepan.merta@seznam.cz" class="contact-link">"stepan.merta@seznam.cz"</a>
                </div>
            </div>
            <div class="main-nav">
                <MenuToggle label=t("nav.open_menu")/>
                <div class="logo">
                    <a aria-label=t("nav.domu") href=path("/")>
                        <img alt=t("site.name") class="logo" src=asset("logo-wm.webp")/>
                    </a>
                </div>
                <nav class="nav-links desktop-nav">
                    <NavLinks/>
                </nav>
                <div class="header-controls">
                    <ThemeToggle id="themeToggle"/>
                    <AccountButton user=user/>
                    <CartWidget cart=cart texts=WidgetTexts::translated()/>
                    <LanguageSwitch switch_cs=switch_cs switch_de=switch_de/>
                </div>
            </div>
        </header>
    }
}

/// The main menu, in the header and in the mobile sidebar.
#[component]
pub fn NavLinks() -> impl IntoView {
    let here = current_path();
    let on_contact_page = here == "/kontakt";
    let link = move |target: &str, key: &str| {
        view! { <a class="nav-link" class:active=here == target href=path(target)>{t(key)}</a> }
    };
    view! {
        {link("/", "nav.domu")}
        {link("/eshop", "nav.eshop")}
        {link("/sortiment", "nav.sortiment")}
        {link("/sluzby", "nav.sluzby")}
        <ContactNavLinks links=ContactLinks {
            on_contact_page,
            cenik: t("nav.cenik"),
            cenik_href: path("/kontakt#cenik"),
            kontakt: t("nav.kontakt"),
            kontakt_href: path("/kontakt"),
        }/>
    }
}

/// The light/dark switch.
#[component]
pub fn ThemeToggle(id: &'static str) -> impl IntoView {
    view! { <page::ThemeToggle id=id.to_string() label=t("nav.toggle_theme")/> }
}

/// The account page, or signing in; the profile picture when there is one.
#[component]
fn AccountButton(user: Option<UserBadge>) -> impl IntoView {
    let href = path(if user.is_some() { "/muj-ucet" } else { "/prihlaseni" });
    let avatar = user.and_then(|user| user.avatar_path);
    view! {
        <a class="btn-pill account-btn" aria-label=t("nav.account") href=href>
            {match avatar {
                Some(src) => {
                    view! { <img alt=t("auth.avatar_alt") class="account-btn-avatar" width="22" height="22" src=src/> }
                        .into_any()
                }
                None => {
                    view! {
                        <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                            <circle cx="12" cy="7" r="4"/>
                        </svg>
                    }
                        .into_any()
                }
            }}
        </a>
    }
}

/// Czech (crowns) or Austrian German (euros).
#[component]
fn LanguageSwitch(switch_cs: String, switch_de: String) -> impl IntoView {
    let german = locale() == Locale::De;
    view! {
        <div class="lang-dropdown">
            <button class="btn-pill lang-btn">
                {if german { view! { <FlagAt/> }.into_any() } else { view! { <FlagCz/> }.into_any() }}
                " "
                <span class="lang-currency">{if german { "EUR" } else { "CZK" }}</span>
                " "
                <span class="chevron">"▼"</span>
            </button>
            <div class="lang-dropdown-content">
                <a class="lang-option" class:active=!german href=switch_cs>
                    <FlagCz/>
                    <span class="lang-currency">"CZK"</span>
                </a>
                " "
                <a class="lang-option" class:active=german href=switch_de>
                    <FlagAt/>
                    <span class="lang-currency">"EUR"</span>
                </a>
            </div>
        </div>
    }
}
