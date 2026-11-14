//! The parts every page shares: header, breadcrumbs, messages, footer,
//! sticky logo, mobile menu and cookie banner.

use crate::i18n::{Locale, t};
use crate::islands::{Dropdown, PageBehaviour, ThemeToggle};
use crate::visitor::{Flash, Visitor, visitor};
use leptos::prelude::*;

const PHONE_ICON: &str = r#"<path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"></path>"#;
const MAIL_ICON: &str = r#"<path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path><polyline points="22,6 12,13 2,6"></polyline>"#;

pub const PHONE: &str = "+420 602 446 339";
pub const PHONE_HREF: &str = "tel:+420602446339";
pub const PHONE_2: &str = "+420 515 235 527";
pub const PHONE_2_HREF: &str = "tel:+420515235527";
pub const EMAIL: &str = "stepan.merta@seznam.cz";

/// The page frame: everything around a page's own content.
#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let v = visitor();
    view! {
        <div class="page-wrapper">
            <Header v=v.clone()/>
            <Breadcrumbs v=v.clone()/>
            {v.flash.clone().map(|flash| match flash {
                Flash::Notice(text) => view! { <div class="page-flash page-flash-notice">{text}</div> },
                Flash::Alert(text) => view! { <div class="page-flash page-flash-alert">{text}</div> },
            })}
            {children()}
            <Footer locale=v.locale/>
        </div>
        <StickyLogo v=v.clone()/>
        <MobileSidebar v=v.clone()/>
        <CookieConsent v=v.clone()/>
        <div id="cart-added-modal"></div>
        <PageBehaviour/>
    }
}

#[component]
fn ContactItem(
    icon: &'static str,
    size: u32,
    href: &'static str,
    text: &'static str,
) -> impl IntoView {
    view! {
        <div class="contact-item">
            <svg width=size height=size fill="none" stroke="#30b109" stroke-width="2" viewBox="0 0 24 24" inner_html=icon></svg>
            <a href=href class="contact-link">{text}</a>
        </div>
    }
}

/// The main navigation links (header and mobile sidebar).
#[component]
fn NavLinks(v: Visitor) -> impl IntoView {
    let l = v.locale;
    let link = |path: &'static str, text: &'static str| {
        let class = if v.is_at(path) {
            "nav-link active"
        } else {
            "nav-link"
        };
        view! { <a href=v.link(path) class=class>{text}</a> }
    };
    view! {
        {link("/", t::nav::domu(l))}
        {link("/eshop", t::nav::eshop(l))}
        {link("/sortiment", t::nav::sortiment(l))}
        {link("/sluzby", t::nav::sluzby(l))}
        <a href=v.link("/kontakt#cenik") class="nav-link" id="nav-cenik">{t::nav::cenik(l)}</a>
        <a href=v.link("/kontakt") class="nav-link" id="nav-kontakt">{t::nav::kontakt(l)}</a>
    }
}

#[component]
fn Header(v: Visitor) -> impl IntoView {
    let l = v.locale;
    let account_href = v.link(if v.user.is_some() {
        "/muj-ucet"
    } else {
        "/prihlaseni"
    });
    view! {
        <header class="site-header">
            <div class="contact-header">
                <ContactItem icon=PHONE_ICON size=14 href=PHONE_HREF text=PHONE/>
                <ContactItem icon=MAIL_ICON size=14 href="mailto:stepan.merta@seznam.cz" text=EMAIL/>
            </div>
            <div class="main-nav">
                <button class="menu-toggle" id="menuToggle" aria-label=t::nav::open_menu(l)>
                    <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M3 12h18M3 6h18M3 18h18"></path></svg>
                </button>
                <div class="logo">
                    <a href=v.link("/") aria-label=t::nav::domu(l)>
                        <img src="/img/logo-wm.webp" alt=t::site::name(l) class="logo" loading="eager"/>
                    </a>
                </div>
                <nav class="nav-links desktop-nav">
                    <NavLinks v=v.clone()/>
                </nav>
                <div class="header-controls">
                    <ThemeToggle id="themeToggle".to_owned() label=t::nav::toggle_theme(l).to_owned() signed_in=v.user.is_some()/>
                    <a href=account_href class="btn-pill account-btn" aria-label=t::nav::account(l)>
                        <AccountIcon v=v.clone()/>
                    </a>
                    <CartWidget v=v.clone()/>
                    <LanguageSwitch v=v.clone()/>
                </div>
            </div>
        </header>
    }
}

/// The signed-in user's profile picture, or a person outline.
#[component]
fn AccountIcon(v: Visitor) -> impl IntoView {
    match v.user.and_then(|u| u.avatar) {
        Some(src) => view! {
            <img src=src alt=t::auth::avatar_alt(v.locale) class="account-btn-avatar" width="22" height="22"/>
        }
        .into_any(),
        None => view! {
            <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path><circle cx="12" cy="7" r="4"></circle></svg>
        }
        .into_any(),
    }
}

#[component]
pub fn FlagCz() -> impl IntoView {
    view! {
        <svg width="20" height="15" viewBox="0 0 600 400" xmlns="http://www.w3.org/2000/svg" class="flag-icon" style="border-radius: 2px;">
            <rect width="600" height="400" fill="#d7141a" class="flag-red"/>
            <rect width="600" height="200" fill="#fff"/>
            <polygon points="0,0 0,400 300,200" fill="#11457e"/>
        </svg>
    }
}

#[component]
pub fn FlagAt() -> impl IntoView {
    view! {
        <svg width="20" height="15" viewBox="0 0 600 400" xmlns="http://www.w3.org/2000/svg" class="flag-icon" style="border-radius: 2px;">
            <rect width="600" height="400" fill="#ed2939" class="flag-red"/>
            <rect width="600" height="133.33" y="133.33" fill="#fff"/>
        </svg>
    }
}

fn flag(locale: Locale) -> AnyView {
    match locale {
        Locale::Cs => view! { <FlagCz/> }.into_any(),
        Locale::De => view! { <FlagAt/> }.into_any(),
    }
}

#[component]
fn LanguageSwitch(v: Visitor) -> impl IntoView {
    let l = v.locale;
    let option = |locale: Locale| {
        let class = if locale == l {
            "lang-option active"
        } else {
            "lang-option"
        };
        view! {
            <a href=v.switch_language_link(locale) class=class>
                {flag(locale)}<span class="lang-currency">{locale.currency()}</span>
            </a>
        }
    };
    let (cs, de, current) = (option(Locale::Cs), option(Locale::De), flag(l));
    let label = t::nav::language_currency(l);
    view! {
        <Dropdown class="lang-dropdown".to_owned()>
            <button type="button" class="btn-pill lang-btn" aria-haspopup="true" aria-expanded="false" aria-label=label>
                {current}
                <span class="lang-currency">{l.currency()}</span>
                <span class="chevron">"▼"</span>
            </button>
            <div class="lang-dropdown-content">
                {cs}
                {de}
            </div>
        </Dropdown>
    }
}

#[component]
fn CartWidget(v: Visitor) -> impl IntoView {
    // Worked out on the server; the island keeps it current in the browser.
    #[cfg(feature = "ssr")]
    let summary = crate::cart_view::summary(&v);
    #[cfg(not(feature = "ssr"))]
    let summary = {
        let _ = v;
        crate::cart_view::CartSummary::default()
    };
    view! { <crate::cart_islands::CartWidget summary=summary/> }
}

/// The crumbs come from the address: /sortiment/palivove-drevo → Sortiment › Palivové dřevo.
fn crumb_name(l: Locale, segment: &str) -> String {
    let known = match segment {
        "sortiment" => Some(t::breadcrumbs::sortiment(l)),
        "palivove-drevo" => Some(t::breadcrumbs::palivove_drevo(l)),
        "stavebni-rezivo" => Some(t::breadcrumbs::stavebni_rezivo(l)),
        "truhlarske-rezivo" => Some(t::breadcrumbs::truhlarske_rezivo(l)),
        "okrasne-kamenivo" => Some(t::breadcrumbs::okrasne_kamenivo(l)),
        "vyrobni-zbytky" => Some(t::breadcrumbs::vyrobni_zbytky(l)),
        "sluzby" => Some(t::breadcrumbs::sluzby(l)),
        "kontakt" => Some(t::breadcrumbs::kontakt(l)),
        "eshop" => Some(t::breadcrumbs::eshop(l)),
        "kosik" => Some(t::cart::page_title(l)),
        "obchodni-podminky" => Some(t::legal::terms_title(l)),
        "zasady-cookies" => Some(t::legal::cookies_title(l)),
        "zasady-ochrany-osobnich-udaju" => Some(t::legal::privacy_title(l)),
        "prihlaseni" => Some(t::auth::login_title(l)),
        "registrace" => Some(t::auth::register_title(l)),
        "zapomenute-heslo" => Some(t::auth::reset::forgot_title(l)),
        "nove-heslo" => Some(t::auth::reset::new_title(l)),
        "muj-ucet" => Some(t::auth::account_title(l)),
        "upravit" => Some(t::auth::edit_profile_title(l)),
        _ => None,
    };
    known.map_or_else(
        || {
            let text = segment.replace('-', " ");
            let mut chars = text.chars();
            chars
                .next()
                .map(|c| c.to_uppercase().chain(chars).collect())
                .unwrap_or_default()
        },
        str::to_owned,
    )
}

/// A page can set its own crumbs (a product page) with this context.
#[derive(Clone, Debug)]
pub struct Crumbs(pub Vec<(String, String)>);

#[component]
fn Breadcrumbs(v: Visitor) -> impl IntoView {
    let l = v.locale;
    let items = use_context::<Crumbs>().map(|c| c.0).unwrap_or_else(|| {
        let mut path = String::new();
        crate::i18n::strip_prefix(&v.path)
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|segment| {
                path.push('/');
                path.push_str(segment);
                (crumb_name(l, segment), path.clone())
            })
            .collect()
    });
    let last = items.len().saturating_sub(1);
    (!items.is_empty()).then(|| {
        view! {
            <nav aria-label=t::breadcrumbs::home_aria(l) class="breadcrumb-nav">
                <a href=v.link("/") class="breadcrumb-home">
                    <svg viewBox="0 0 24 24" width="24" height="24" fill="#5890FF"><path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/></svg>
                </a>
                {items.into_iter().enumerate().map(|(i, (name, path))| view! {
                    <svg class="breadcrumb-separator" viewBox="0 0 24 24" width="20" height="20" fill="#000000"><path d="M8.59 16.59L13.17 12 8.59 7.41 10 6l6 6-6 6-1.41-1.41z"/></svg>
                    <a href=v.link(&path) class=if i == last { "breadcrumb-item active" } else { "breadcrumb-item" }>{name}</a>
                }).collect_view()}
            </nav>
        }
    })
}

#[component]
fn Footer(locale: Locale) -> impl IntoView {
    let l = locale;
    let link = |path: &str| l.path(path);
    view! {
        <footer class="site-footer">
            <div class="footer-container">
                <div class="footer-top">
                    <div class="footer-col brand-col">
                        <img src="/img/logo-dm.webp" alt=t::site::name(l) class="footer-logo" loading="lazy"/>
                        <h2>{t::footer::brand_name(l)}</h2>
                        <span class="brand-blue">{t::footer::brand_sub(l)}</span>
                    </div>
                    <div class="footer-col">
                        <h3>{t::footer::kontakt_title(l)}</h3>
                        <ul class="contact-list">
                            <li>
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="2" width="14" height="20" rx="2" ry="2"></rect><line x1="12" y1="18" x2="12.01" y2="18"></line></svg>
                                <a href=PHONE_HREF class="contact-link">{PHONE}</a>
                            </li>
                            <li>
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" inner_html=PHONE_ICON></svg>
                                <a href=PHONE_2_HREF class="contact-link">{PHONE_2}</a>
                            </li>
                            <li>
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" inner_html=MAIL_ICON></svg>
                                <a href="mailto:stepan.merta@seznam.cz" class="contact-link">{EMAIL}</a>
                            </li>
                            <li>
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"></path><circle cx="12" cy="10" r="3"></circle></svg>
                                {t::footer::address(l)}
                            </li>
                            <li>
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
                                {t::footer::hours(l)}
                            </li>
                        </ul>
                    </div>
                    <div class="footer-col">
                        <h3>{t::footer::sluzby_title(l)}</h3>
                        <ul class="link-list">
                            <li><a href=link("/kontakt#kalkulace")>{t::footer::sluzby_links::zakazkova_vyroba(l)}</a></li>
                            <li><a href=link("/sluzby#hoblovani")>{t::footer::sluzby_links::hoblovani(l)}</a></li>
                            <li><a href=link("/sluzby#doprava")>{t::footer::sluzby_links::doprava(l)}</a></li>
                            <li><a href=link("/sluzby#impregnace")>{t::footer::sluzby_links::impregnace(l)}</a></li>
                            <li><a href=link("/sluzby#porez-prodej-kulatiny")>{t::footer::sluzby_links::porez(l)}</a></li>
                        </ul>
                    </div>
                    <div class="footer-col">
                        <h3>{t::footer::sortiment_title(l)}</h3>
                        <ul class="link-list">
                            <li><a href=link("/sortiment/palivove-drevo")>{t::footer::sortiment_links::palivove_drevo(l)}</a></li>
                            <li><a href=link("/sortiment/stavebni-rezivo")>{t::footer::sortiment_links::stavebni_rezivo(l)}</a></li>
                            <li><a href=link("/sortiment/truhlarske-rezivo")>{t::footer::sortiment_links::truhlarske_rezivo(l)}</a></li>
                            <li><a href=link("/sortiment/vyrobni-zbytky")>{t::footer::sortiment_links::vyrobni_zbytky(l)}</a></li>
                            <li><a href=link("/sortiment/okrasne-kamenivo")>{t::footer::sortiment_links::okrasne_kamenivo(l)}</a></li>
                        </ul>
                    </div>
                    <div class="footer-col social-col">
                        <a href="https://www.facebook.com/piladerflice/?locale=cs_CZ" target="_blank" rel="noopener" aria-label="Facebook">
                            <svg viewBox="0 0 24 24" fill="#5c8df6"><path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h7.3v-6.9h-2V11.2h2V9.3c0-2.8 1.3-4.3 4.3-4.3 1.1 0 2.5.2 2.5.2v2.7h-1.4c-1.4 0-1.6.8-1.6 1.7v1.6h2.9l-.4 2.9h-2.5V21H19c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2z"/></svg>
                        </a>
                    </div>
                </div>
                <div class="footer-bottom">
                    <a href=link("/zasady-ochrany-osobnich-udaju") class="footer-cookie-settings">{t::legal::privacy_title(l)}</a>
                    <button type="button" class="footer-cookie-settings" data-cookie-settings="">{t::cookies::settings_link(l)}</button>
                </div>
                <div class="footer-map">
                    <iframe
                        src="https://www.google.com/maps/embed?pb=!1m18!1m12!1m3!1d10510.197962898043!2d16.121110787158198!3d48.80957630000002!2m3!1f0!2f0!3f0!3m2!1i1024!2i768!4f13.1!3m3!1m2!1s0x476d5449f74dd161%3A0xe25da7a72d478ac5!2sPila%20Derflice%20-%20%C5%A0t%C4%9Bp%C3%A1n%20Merta%20-%20Prodej%20D%C5%99eva%20Znojmo!5e0!3m2!1scs!2scz!4v1783167177644!5m2!1scs!2scz"
                        width="100%"
                        height="350"
                        style="border:0;"
                        allowfullscreen=""
                        loading="lazy"
                        referrerpolicy="no-referrer-when-downgrade"
                    ></iframe>
                </div>
            </div>
        </footer>
    }
}

#[component]
fn StickyLogo(v: Visitor) -> impl IntoView {
    view! {
        <div class="sticky-logo-bar" id="stickyLogoBar">
            <a href=v.link("/") aria-label=t::nav::domu(v.locale)>
                <img src="/img/logo-wm.webp" alt=t::site::name(v.locale) class="sticky-logo-img" loading="lazy"/>
            </a>
        </div>
    }
}

#[component]
fn MobileSidebar(v: Visitor) -> impl IntoView {
    let l = v.locale;
    let signed_in = v.user.is_some();
    let account_class = if v.is_at("/muj-ucet") || v.is_at("/prihlaseni") {
        "sidebar-account active"
    } else {
        "sidebar-account"
    };
    view! {
        <div class="sidebar-overlay" id="sidebarOverlay"></div>
        <aside class="mobile-sidebar" id="mobileSidebar">
            <div class="sidebar-header">
                <a href=v.link("/") aria-label=t::nav::domu(l)>
                    <img src="/img/logo-wm.webp" alt=t::site::name(l) class="sidebar-logo" loading="lazy"/>
                </a>
                <button class="close-sidebar" id="closeSidebar" aria-label=t::nav::close_menu(l)>"✕"</button>
            </div>
            <nav class="sidebar-nav">
                <NavLinks v=v.clone()/>
            </nav>
            <div class="sidebar-bottom">
                <div class="sidebar-theme-toggle">
                    <span>{t::nav::dark_mode_label(l)}</span>
                    <ThemeToggle id="themeToggleMobile".to_owned() label=t::nav::toggle_theme(l).to_owned() signed_in=signed_in/>
                </div>
                <div class="sidebar-contact">
                    <ContactItem icon=PHONE_ICON size=18 href=PHONE_HREF text=PHONE/>
                    <ContactItem icon=MAIL_ICON size=14 href="mailto:stepan.merta@seznam.cz" text=EMAIL/>
                </div>
                <a href=v.link(if signed_in { "/muj-ucet" } else { "/prihlaseni" }) class=account_class>
                    <AccountIcon v=v.clone()/>
                    <span>{if signed_in { t::nav::account(l) } else { t::nav::login(l) }}</span>
                </a>
            </div>
        </aside>
    }
}

#[component]
fn CookieConsent(v: Visitor) -> impl IntoView {
    let l = v.locale;
    view! {
        <div id="cookie-consent-banner" class="cookie-consent-banner" hidden>
            <div class="cookie-consent-content">
                <p class="cookie-consent-text">
                    {t::cookies::banner_text(l)}" "
                    <a href=v.link("/zasady-cookies") class="cookie-consent-link">{t::cookies::learn_more(l)}</a>
                </p>
                <div class="cookie-consent-actions">
                    <button type="button" class="btn-cookie btn-cookie-decline" data-cookie-consent="declined">{t::cookies::decline(l)}</button>
                    <button type="button" class="btn-cookie btn-cookie-necessary" data-cookie-consent="necessary">{t::cookies::necessary_only(l)}</button>
                    <button type="button" class="btn-cookie btn-cookie-accept" data-cookie-consent="all">{t::cookies::accept_all(l)}</button>
                </div>
            </div>
        </div>
    }
}
