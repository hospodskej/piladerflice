use crate::i18n::t;
use crate::pages::cart::CartPage;
use crate::pages::checkout::{ConfirmationPage, DetailsPage, ShippingPage, SummaryPage};
use crate::pages::eshop::{EshopPage, ProductPage};
use crate::pages::home::HomePage;
use crate::pages::info::*;
use crate::pages::kontakt::KontaktPage;
use crate::pages::sortiment::*;
use crate::visitor::visitor;
use leptos::nonce::use_nonce;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    path, SsrMode,
};

/// The Google Analytics property.
const GA_MEASUREMENT_ID: &str = "G-CMJ46E7F3G";

/// Applies the saved dark/light choice before the first paint: the account's
/// choice when signed in, else this device's.
const THEME_SCRIPT: &str = r#"(function () {
  var saved = document.querySelector('meta[name="theme-preference"]');
  var theme = saved && saved.content;
  try {
    if (theme) localStorage.setItem("theme", theme);
    else theme = localStorage.getItem("theme");
  } catch (e) {}
  if (theme === "dark") document.documentElement.classList.add("dark-mode");
})();"#;

/// Analytics stays off until the visitor accepts all cookies.
const CONSENT_SCRIPT: &str = r#"window.dataLayer = window.dataLayer || [];
function gtag() { dataLayer.push(arguments); }
var storedConsent = (document.cookie.match(/(?:^|; )cookie_consent=([^;]*)/) || [])[1];
gtag("consent", "default", {
  ad_storage: "denied",
  ad_user_data: "denied",
  ad_personalization: "denied",
  analytics_storage: storedConsent === "all" ? "granted" : "denied"
});"#;

/// The HTML document around every page.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    let v = visitor();
    let l = v.locale;
    let theme = v.user.as_ref().map(|u| u.theme.map(|t| t.as_str()).unwrap_or_default());
    let nonce = use_nonce();
    view! {
        <!DOCTYPE html>
        <html lang=l.as_str()>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width,initial-scale=1"/>
                <meta name="apple-mobile-web-app-capable" content="yes"/>
                <meta name="application-name" content="Pila Derflice"/>
                <meta name="mobile-web-app-capable" content="yes"/>
                {theme.map(|theme| view! { <meta name="theme-preference" content=theme/> })}
                <script nonce=nonce.clone()>{THEME_SCRIPT}</script>
                <script nonce=nonce.clone()>{CONSENT_SCRIPT}</script>
                <script async nonce=nonce.clone() src=format!("https://www.googletagmanager.com/gtag/js?id={GA_MEASUREMENT_ID}")></script>
                <script nonce=nonce>{format!(r#"gtag("js", new Date()); gtag("config", "{GA_MEASUREMENT_ID}");"#)}</script>
                <link rel="icon" href="/icon.svg" type="image/svg+xml"/>
                <link rel="stylesheet" href=crate::assets::STYLESHEET_HREF/>
                <HydrationScripts options islands=true/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let l = visitor().locale;
    view! {
        <leptos_meta::Title text=t::site::name(l)/>
        <Router>
            <Routes fallback=crate::pages::not_found::NotFoundPage>
                <Pages/>
                <ParentRoute path=path!("/at") view=leptos_router::components::Outlet>
                    <Pages/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}

/// Every public page; mounted at the root (Czech) and under /at (German).
#[component(transparent)]
fn Pages() -> impl leptos_router::MatchNestedRoutes + Clone {
    // One parent around them all, so this is a single route whether or not
    // the dev build erases component types.
    view! {
        <ParentRoute path=path!("") view=leptos_router::components::Outlet>
        <Route path=path!("") view=HomePage ssr=SsrMode::Async/>
        <Route path=path!("/kontakt") view=KontaktPage ssr=SsrMode::Async/>
        <Route path=path!("/eshop") view=EshopPage ssr=SsrMode::Async/>
        <Route path=path!("/kosik") view=CartPage ssr=SsrMode::Async/>
        <Route path=path!("/kosik/doprava") view=ShippingPage ssr=SsrMode::Async/>
        <Route path=path!("/kosik/udaje") view=DetailsPage ssr=SsrMode::Async/>
        <Route path=path!("/kosik/souhrn") view=SummaryPage ssr=SsrMode::Async/>
        <Route path=path!("/kosik/dekujeme") view=ConfirmationPage ssr=SsrMode::Async/>
        <Route path=path!("/eshop/:key") view=ProductPage ssr=SsrMode::Async/>
        <Route path=path!("/sluzby") view=SluzbyPage ssr=SsrMode::Async/>
        <Route path=path!("/obchodni-podminky") view=TermsPage/>
        <Route path=path!("/zasady-cookies") view=CookiesPage/>
        <Route path=path!("/zasady-ochrany-osobnich-udaju") view=PrivacyPage/>
        <Route path=path!("/sortiment") view=SortimentPage ssr=SsrMode::Async/>
        <Route path=path!("/sortiment/palivove-drevo") view=PalivoveDrevoPage/>
        <Route path=path!("/sortiment/stavebni-rezivo") view=StavebniRezivoPage/>
        <Route path=path!("/sortiment/truhlarske-rezivo") view=TruhlarskeRezivoPage/>
        <Route path=path!("/sortiment/okrasne-kamenivo") view=OkrasneKamenivoPage/>
        <Route path=path!("/sortiment/vyrobni-zbytky") view=VyrobniZbytkyPage/>
        </ParentRoute>
    }
    .into_inner()
}
