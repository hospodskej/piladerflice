//! The page around every page's content: `<head>`, the header with the
//! menu, account, cart and language, the breadcrumbs, the message of the
//! last form, the footer, the mobile menu and the cookie banner.

mod data;
mod footer;
mod head;
mod header;
mod sidebar;

pub use data::{GA_MEASUREMENT_ID, LayoutData};

use leptos::prelude::*;

use super::{asset, path, t};
use crate::islands::cart::{CartAddedModal, ModalTexts};
use crate::islands::page::{self, CookieTexts};
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
        pkg_path,
        dark_mode,
        consent,
    } = data;
    let banner = consent.is_none();
    // The theme's class only when dark (a `class` in the view would
    // always be written, if empty).
    let theme = dark_mode.then_some("dark-mode");
    view! {
        <html lang=locale.code()>
            <head::Head head=head stylesheets=stylesheets pkg_path=pkg_path consent=consent/>
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
                <CookieBanner show=banner/>
                <CartAddedModal texts=ModalTexts::translated()/>
            </body>
        </html>
    }
    .attr("class", theme)
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
        <page::StickyLogo>
            <a aria-label=t("nav.domu") href=path("/")>
                <img alt=t("site.name") class="sticky-logo-img" src=asset("logo-wm.webp")/>
            </a>
        </page::StickyLogo>
    }
}

#[component]
fn CookieBanner(show: bool) -> impl IntoView {
    let texts = CookieTexts {
        text: t("cookies.banner_text"),
        learn_more: t("cookies.learn_more"),
        learn_more_href: path("/zasady-cookies"),
        decline: t("cookies.decline"),
        necessary_only: t("cookies.necessary_only"),
        accept_all: t("cookies.accept_all"),
    };
    view! { <page::CookieBanner show=show texts=texts/> }
}
