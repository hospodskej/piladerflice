//! The page's `<head>`: title, search engine and share links, scripts and
//! stylesheets.

use leptos::prelude::*;

use super::GA_MEASUREMENT_ID;
use super::data::{Head as HeadData, SearchLinks, ShareCard};
use crate::ui::asset;

#[component]
pub fn Head(head: HeadData, stylesheets: Vec<String>, importmap_json: String, preloads: Vec<String>) -> impl IntoView {
    let HeadData {
        title,
        site_name,
        description,
        links,
        noindex,
        share,
        json_ld,
    } = head;
    view! {
        <head>
            <title>{title.clone()}</title>
            {description.clone().map(|description| view! { <meta name="description" content=description/> })}
            {links.map(|links| view! { <LanguageLinks links=links/> })}
            <ShareMeta share=share title=title site_name=site_name description=description/>
            <meta name="viewport" content="width=device-width,initial-scale=1"/>
            <meta name="apple-mobile-web-app-capable" content="yes"/>
            <meta name="application-name" content="Pila Derflice"/>
            <meta name="mobile-web-app-capable" content="yes"/>
            <script inner_html=THEME_SCRIPT></script>
            <script inner_html=CONSENT_SCRIPT></script>
            <script async src=format!("https://www.googletagmanager.com/gtag/js?id={GA_MEASUREMENT_ID}")></script>
            <script inner_html=format!("gtag(\"js\", new Date());gtag(\"config\", \"{GA_MEASUREMENT_ID}\");")></script>
            {noindex.then(|| view! { <meta name="robots" content="noindex"/> })}
            {json_ld.map(|json| view! { <script type="application/ld+json" inner_html=json></script> })}
            <link rel="icon" type="image/x-icon" href=asset("icon.svg")/>
            {stylesheets
                .into_iter()
                .map(|href| view! { <link rel="stylesheet" href=href data-turbo-track="reload"/> })
                .collect_view()}
            <script type="importmap" data-turbo-track="reload" inner_html=importmap_json></script>
            {preloads.into_iter().map(|href| view! { <link rel="modulepreload" href=href/> }).collect_view()}
            <script type="module" inner_html=IMPORT_APPLICATION></script>
        </head>
    }
}

const IMPORT_APPLICATION: &str = "import \"application\"";

const THEME_SCRIPT: &str = r#"(function () { try { if (localStorage.getItem("theme") === "dark") { document.documentElement.classList.add("dark-mode"); } } catch (e) {} })();"#;

const CONSENT_SCRIPT: &str = r#"window.dataLayer = window.dataLayer || []; function gtag() { dataLayer.push(arguments); } var storedConsent = (document.cookie.match(/(?:^|; )cookie_consent=([^;]*)/) || [])[1]; gtag("consent", "default", { ad_storage: "denied", ad_user_data: "denied", ad_personalization: "denied", analytics_storage: storedConsent === "all" ? "granted" : "denied" });"#;

/// Where search engines find this page in each language.
#[component]
fn LanguageLinks(links: SearchLinks) -> impl IntoView {
    view! {
        <link rel="canonical" href=links.canonical/>
        {links
            .alternates
            .into_iter()
            .map(|(hreflang, href)| view! { <link rel="alternate" hreflang=hreflang href=href/> })
            .collect_view()}
        <link rel="alternate" hreflang="x-default" href=links.x_default/>
    }
}

/// `<meta property="og:..." content="...">` (`property` is set by hand:
/// Leptos doesn't list it for meta).
fn og(property: &'static str, content: impl Into<String>) -> impl IntoView {
    leptos::html::meta().attr("property", property).content(content.into())
}

/// The preview card shown when a link is shared (WhatsApp, Facebook, iMessage...).
#[component]
fn ShareMeta(share: ShareCard, title: String, site_name: String, description: Option<String>) -> impl IntoView {
    view! {
        {og("og:type", "website")}
        {og("og:site_name", site_name.clone())}
        {og("og:title", title)}
        {og("og:url", share.url)}
        {og("og:locale", share.locale)}
        {share
            .alternate_locales
            .into_iter()
            .map(|locale| og("og:locale:alternate", locale))
            .collect_view()}
        {description.map(|description| og("og:description", description))}
        {match share.image {
            Some(image) => {
                view! {
                    {og("og:image", image.url)}
                    {og("og:image:type", image.mime)}
                    {og("og:image:width", "1200")}
                    {og("og:image:height", "630")}
                    {og("og:image:alt", site_name)}
                    <meta name="twitter:card" content="summary_large_image"/>
                }
                    .into_any()
            }
            None => view! { <meta name="twitter:card" content="summary"/> }.into_any(),
        }}
    }
}
