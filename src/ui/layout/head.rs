//! The page's `<head>`: title, search engine and share links, scripts and
//! stylesheets.

use leptos::config::LeptosOptions;
use leptos::hydration::HydrationScripts;
use leptos::prelude::*;

use super::GA_MEASUREMENT_ID;
use super::data::{Head as HeadData, SearchLinks, ShareCard};
use crate::ui::asset;
use crate::wasm::OUTPUT_NAME;

#[component]
pub fn Head(
    head: HeadData,
    stylesheets: Vec<String>,
    pkg_path: Option<String>,
    consent: Option<String>,
) -> impl IntoView {
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
            <script inner_html=analytics_consent(consent.as_deref())></script>
            <script async src=format!("https://www.googletagmanager.com/gtag/js?id={GA_MEASUREMENT_ID}")></script>
            <script inner_html=format!("gtag(\"js\", new Date());gtag(\"config\", \"{GA_MEASUREMENT_ID}\");")></script>
            {noindex.then(|| view! { <meta name="robots" content="noindex"/> })}
            {json_ld.map(|json| view! { <script type="application/ld+json" inner_html=json></script> })}
            <link rel="icon" type="image/x-icon" href=asset("icon.svg")/>
            {stylesheets
                .into_iter()
                .map(|href| view! { <link rel="stylesheet" href=href/> })
                .collect_view()}
            <IslandScripts pkg_path=pkg_path/>
        </head>
    }
}

/// Google Analytics' consent: denied until the visitor accepts all cookies
/// (the cookie banner island updates it when they choose).
fn analytics_consent(consent: Option<&str>) -> String {
    let analytics = if consent == Some("all") { "granted" } else { "denied" };
    format!(
        r#"window.dataLayer = window.dataLayer || []; function gtag() {{ dataLayer.push(arguments); }} gtag("consent", "default", {{ ad_storage: "denied", ad_user_data: "denied", ad_personalization: "denied", analytics_storage: "{analytics}" }});"#
    )
}

/// Loads the islands' WebAssembly from `/{pkg_path}/` (Leptos' loader);
/// nothing without a build.
#[component]
pub fn IslandScripts(pkg_path: Option<String>) -> impl IntoView {
    pkg_path.map(|pkg_path| {
        let options = LeptosOptions::builder()
            .output_name(OUTPUT_NAME)
            .site_pkg_dir(pkg_path)
            .hash_files(false)
            .build();
        view! { <HydrationScripts options=options islands=true/> }
    })
}

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
