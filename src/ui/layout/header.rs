//! The header: contact line, menu, account, cart and language.

use leptos::prelude::*;

use super::data::UserBadge;
use crate::locale::Locale;
use crate::ui::icons::{CartIcon, FlagAt, FlagCz, MailIcon, PhoneIcon};
use crate::ui::{asset, current_path, locale, path, t};
use crate::views::cart_view::CartView;

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
                <button class="menu-toggle" id="menuToggle" aria-label=t("nav.open_menu")>
                    <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                        <path d="M3 12h18M3 6h18M3 18h18"/>
                    </svg>
                </button>
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
                    <CartWidget cart=cart/>
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
    let link = move |target: &str, key: &str| {
        view! { <a class="nav-link" class:active=here == target href=path(target)>{t(key)}</a> }
    };
    view! {
        {link("/", "nav.domu")}
        {link("/eshop", "nav.eshop")}
        {link("/sortiment", "nav.sortiment")}
        {link("/sluzby", "nav.sluzby")}
        <a class="nav-link" id="nav-cenik" href=path("/kontakt#cenik")>{t("nav.cenik")}</a>
        <a class="nav-link" id="nav-kontakt" href=path("/kontakt")>{t("nav.kontakt")}</a>
    }
}

/// The light/dark switch.
#[component]
pub fn ThemeToggle(id: &'static str) -> impl IntoView {
    view! {
        <button type="button" class="theme-toggle" id=id aria-label=t("nav.toggle_theme")>
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

/// The cart button and the list that opens under it.
#[component]
pub fn CartWidget(cart: CartView) -> impl IntoView {
    view! {
        <div id="cart-widget" class="cart-dropdown">
            <button type="button" class="btn-pill cart-btn" aria-label=t("cart.aria_label")>
                <CartIcon/>
            </button>
            <div class="cart-dropdown-content">
                {if cart.empty {
                    view! { <p class="cart-empty">{t("cart.empty")}</p> }.into_any()
                } else {
                    view! {
                        <ul class="cart-items-list">
                            {cart
                                .lines
                                .into_iter()
                                .map(|item| {
                                    view! {
                                        <li class="cart-item">
                                            {item.image.map(|src| view! { <img alt=item.description.clone() class="cart-item-img" src=src/> })}
                                            <div class="cart-item-details">
                                                <span class="cart-item-desc">{item.description}</span>
                                            </div>
                                            <span class="cart-item-price">{item.total}</span>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                        <div class="cart-totals">
                            <div class="cart-total-row">
                                <span>{t("cart.total_due")}</span>
                                <strong>{cart.subtotal}</strong>
                            </div>
                            <div class="cart-total-row cart-total-excl-vat">
                                <span>{t("cart.without_vat")}</span>
                                <span>{cart.excl_vat}</span>
                            </div>
                        </div>
                        <a class="btn-primary full-width cart-dropdown-view-cart" href=path("/kosik")>{t("cart.view_cart")}</a>
                    }
                        .into_any()
                }}
            </div>
        </div>
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
                <a class="lang-option" class:active=!german data-turbo="false" href=switch_cs>
                    <FlagCz/>
                    <span class="lang-currency">"CZK"</span>
                </a>
                " "
                <a class="lang-option" class:active=german data-turbo="false" href=switch_de>
                    <FlagAt/>
                    <span class="lang-currency">"EUR"</span>
                </a>
            </div>
        </div>
    }
}
