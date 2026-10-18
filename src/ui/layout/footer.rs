//! The footer: contact, links to services and the product range, the map.

use leptos::prelude::*;

use crate::islands::page::CookieSettingsButton;
use crate::ui::{asset, path, t};

const SERVICE_LINKS: [(&str, &str); 5] = [
    ("/kontakt#kalkulace", "footer.sluzby_links.zakazkova_vyroba"),
    ("/sluzby#hoblovani", "footer.sluzby_links.hoblovani"),
    ("/sluzby#doprava", "footer.sluzby_links.doprava"),
    ("/sluzby#impregnace", "footer.sluzby_links.impregnace"),
    ("/sluzby#porez-prodej-kulatiny", "footer.sluzby_links.porez"),
];

const RANGE_LINKS: [(&str, &str); 5] = [
    ("/sortiment/palivove-drevo", "footer.sortiment_links.palivove_drevo"),
    ("/sortiment/stavebni-rezivo", "footer.sortiment_links.stavebni_rezivo"),
    (
        "/sortiment/truhlarske-rezivo",
        "footer.sortiment_links.truhlarske_rezivo",
    ),
    ("/sortiment/vyrobni-zbytky", "footer.sortiment_links.vyrobni_zbytky"),
    ("/sortiment/okrasne-kamenivo", "footer.sortiment_links.okrasne_kamenivo"),
];

const MAP: &str = "https://www.google.com/maps/embed?pb=!1m18!1m12!1m3!1d10510.197962898043!2d16.121110787158198!3d48.80957630000002!2m3!1f0!2f0!3f0!3m2!1i1024!2i768!4f13.1!3m3!1m2!1s0x476d5449f74dd161%3A0xe25da7a72d478ac5!2sPila%20Derflice%20-%20%C5%A0t%C4%9Bp%C3%A1n%20Merta%20-%20Prodej%20D%C5%99eva%20Znojmo!5e0!3m2!1scs!2scz!4v1783167177644!5m2!1scs!2scz";

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="site-footer">
            <div class="footer-container">
                <div class="footer-top">
                    <div class="footer-col brand-col">
                        <img alt=t("site.name") class="footer-logo" src=asset("logo-dm.webp")/>
                        <h2>{t("footer.brand_name")}</h2>
                        <span class="brand-blue">{t("footer.brand_sub")}</span>
                    </div>
                    <div class="footer-col">
                        <h3>{t("footer.kontakt_title")}</h3>
                        <Contacts/>
                    </div>
                    <div class="footer-col">
                        <h3>{t("footer.sluzby_title")}</h3>
                        <LinkList links=SERVICE_LINKS/>
                    </div>
                    <div class="footer-col">
                        <h3>{t("footer.sortiment_title")}</h3>
                        <LinkList links=RANGE_LINKS/>
                    </div>
                    <div class="footer-col social-col">
                        <a href="https://www.facebook.com/piladerflice/?locale=cs_CZ" target="_blank" rel="noopener" aria-label="Facebook">
                            <svg viewBox="0 0 24 24" fill="#5c8df6">
                                <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h7.3v-6.9h-2V11.2h2V9.3c0-2.8 1.3-4.3 4.3-4.3 1.1 0 2.5.2 2.5.2v2.7h-1.4c-1.4 0-1.6.8-1.6 1.7v1.6h2.9l-.4 2.9h-2.5V21H19c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2z"/>
                            </svg>
                        </a>
                    </div>
                </div>
                <div class="footer-bottom">
                    <a class="footer-cookie-settings" href=path("/zasady-ochrany-osobnich-udaju")>{t("legal.privacy_title")}</a>
                    " "
                    <CookieSettingsButton label=t("cookies.settings_link")/>
                </div>
                <div class="footer-map">
                    {map()}
                </div>
            </div>
        </footer>
    }
}

/// The Google map, loaded only when scrolled to (`loading` is set by hand:
/// Leptos doesn't list it for iframes).
fn map() -> impl IntoView {
    leptos::html::iframe()
        .src(MAP)
        .width("100%")
        .height("350")
        .style("border:0;")
        .allowfullscreen("")
        .attr("loading", "lazy")
        .referrerpolicy("no-referrer-when-downgrade")
}

#[component]
fn LinkList(links: [(&'static str, &'static str); 5]) -> impl IntoView {
    view! {
        <ul class="link-list">
            {links.map(|(href, key)| view! { <li><a href=path(href)>{t(key)}</a></li> }).collect_view()}
        </ul>
    }
}

#[component]
fn Contacts() -> impl IntoView {
    view! {
        <ul class="contact-list">
            <li>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="5" y="2" width="14" height="20" rx="2" ry="2"/>
                    <line x1="12" y1="18" x2="12.01" y2="18"/>
                </svg>
                " "
                <a href="tel:+420602446339" class="contact-link">"+420 602 446 339"</a>
            </li>
            <li>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"/>
                </svg>
                " "
                <a href="tel:+420515235527" class="contact-link">"+420 515 235 527"</a>
            </li>
            <li>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/>
                    <polyline points="22,6 12,13 2,6"/>
                </svg>
                " "
                <a href="mailto:stepan.merta@seznam.cz" class="contact-link">"stepan.merta@seznam.cz"</a>
            </li>
            <li>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/>
                    <circle cx="12" cy="10" r="3"/>
                </svg>
                " "
                {t("footer.address")}
            </li>
            <li>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"/>
                    <polyline points="12 6 12 12 16 14"/>
                </svg>
                " "
                {t("footer.hours")}
            </li>
        </ul>
    }
}
