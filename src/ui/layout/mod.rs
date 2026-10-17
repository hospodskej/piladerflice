//! The page around every page's content: `<head>`, the header with the
//! menu, account, cart and language, the breadcrumbs, the message of the
//! last form, the footer, the mobile menu and the cookie banner.

mod data;
mod footer;
mod head;
mod header;
mod sidebar;

pub use data::{GA_MEASUREMENT_ID, LayoutData};
pub use header::CartWidget;

use leptos::prelude::*;

use super::{asset, path, t};
use crate::session::Flash;
use crate::views::Crumb;

#[component]
pub fn Layout(data: LayoutData, children: Children) -> impl IntoView {
    let LayoutData {
        locale,
        head,
        breadcrumbs,
        switch_cs,
        switch_de,
        flash,
        cart,
        user,
        stylesheets,
        importmap_json,
        preloads,
    } = data;
    view! {
        <html lang=locale.code()>
            <head::Head head=head stylesheets=stylesheets importmap_json=importmap_json preloads=preloads/>
            <body>
                <div class="page-wrapper">
                    <header::Header user=user cart=cart switch_cs=switch_cs switch_de=switch_de/>
                    <Breadcrumbs crumbs=breadcrumbs/>
                    {flash.map(|flash| view! { <FlashMessage flash=flash/> })}
                    {children()}
                    <footer::Footer/>
                </div>
                <StickyLogo/>
                <sidebar::MobileSidebar/>
                <CookieBanner/>
                <div id="cart-added-modal"></div>
            </body>
        </html>
    }
}

#[component]
fn FlashMessage(flash: Flash) -> impl IntoView {
    let (kind, text) = match flash {
        Flash::Notice(text) => ("notice", text),
        Flash::Alert(text) => ("alert", text),
    };
    view! { <div class=format!("page-flash page-flash-{kind}")>{text}</div> }
}

/// The trail under the header: home, then each part of the URL.
#[component]
fn Breadcrumbs(crumbs: Vec<Crumb>) -> impl IntoView {
    let last = crumbs.len().saturating_sub(1);
    (!crumbs.is_empty()).then(|| {
        view! {
            <nav aria-label=t("breadcrumbs.home_aria") class="breadcrumb-nav">
                <a class="breadcrumb-home" href=path("/")>
                    <svg viewBox="0 0 24 24" width="24" height="24" fill="#5890FF">
                        <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/>
                    </svg>
                </a>
                {crumbs
                    .into_iter()
                    .enumerate()
                    .map(|(index, crumb)| {
                        view! {
                            <svg class="breadcrumb-separator" viewBox="0 0 24 24" width="20" height="20" fill="#000000">
                                <path d="M8.59 16.59L13.17 12 8.59 7.41 10 6l6 6-6 6-1.41-1.41z"/>
                            </svg>
                            <a class="breadcrumb-item" class:active=index == last href=crumb.href>{crumb.name}</a>
                        }
                    })
                    .collect_view()}
            </nav>
        }
    })
}

/// The logo that stays at the top once the header has scrolled away.
#[component]
fn StickyLogo() -> impl IntoView {
    view! {
        <div class="sticky-logo-bar" id="stickyLogoBar">
            <a aria-label=t("nav.domu") href=path("/")>
                <img alt=t("site.name") class="sticky-logo-img" src=asset("logo-wm.webp")/>
            </a>
        </div>
    }
}

#[component]
fn CookieBanner() -> impl IntoView {
    view! {
        <div id="cookie-consent-banner" class="cookie-consent-banner" hidden>
            <div class="cookie-consent-content">
                <p class="cookie-consent-text">
                    {t("cookies.banner_text")}
                    " "
                    <a class="cookie-consent-link" href=path("/zasady-cookies")>{t("cookies.learn_more")}</a>
                </p>
                <div class="cookie-consent-actions">
                    <button type="button" class="btn-cookie btn-cookie-decline" data-cookie-consent="declined">
                        {t("cookies.decline")}
                    </button>
                    " "
                    <button type="button" class="btn-cookie btn-cookie-necessary" data-cookie-consent="necessary">
                        {t("cookies.necessary_only")}
                    </button>
                    " "
                    <button type="button" class="btn-cookie btn-cookie-accept" data-cookie-consent="all">
                        {t("cookies.accept_all")}
                    </button>
                </div>
            </div>
        </div>
    }
}
