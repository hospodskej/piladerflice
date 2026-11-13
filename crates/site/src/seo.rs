//! Data for search engines.

use crate::i18n::t;
use crate::layout::{EMAIL, PHONE_HREF, PHONE_2_HREF};
use crate::pages::base_url;
use crate::visitor::visitor;
use leptos::prelude::*;

/// schema.org LocalBusiness (JSON-LD). The same facts as the footer and the
/// contact page; keep them in sync. No aggregateRating: Google doesn't accept
/// a business's own ratings, and the Google reviews are on Google already.
#[component]
pub fn LocalBusinessSchema() -> impl IntoView {
    let l = visitor().locale;
    let root = format!("{}{}", base_url(), l.path("/"));
    let logo = format!("{}/img/logo-wm.webp", base_url());
    let data = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "LocalBusiness",
        "@id": format!("{root}#business"),
        "name": t::site::name(l),
        "legalName": "Štěpán Merta",
        "description": t::meta::home::description(l),
        "url": root,
        "logo": logo,
        "image": logo,
        "telephone": [PHONE_HREF.trim_start_matches("tel:"), PHONE_2_HREF.trim_start_matches("tel:")],
        "email": EMAIL,
        "vatID": "CZ6909094742",
        "taxID": "45665451",
        "address": {
            "@type": "PostalAddress",
            "streetAddress": "Derflice 66",
            "postalCode": "671 28",
            "addressLocality": "Znojmo",
            "addressCountry": "CZ"
        },
        "geo": { "@type": "GeoCoordinates", "latitude": 48.8096, "longitude": 16.1211 },
        "openingHoursSpecification": [{
            "@type": "OpeningHoursSpecification",
            "dayOfWeek": ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"],
            "opens": "07:00",
            "closes": "15:30"
        }],
        "areaServed": [t::meta::area_served::south_moravia(l), t::meta::area_served::north_austria(l)],
        "sameAs": ["https://www.facebook.com/piladerflice/"]
    });
    // `<`, `>` and `&` escaped, so nothing in the data can end the script early.
    let json = data.to_string().replace('<', "\\u003c").replace('>', "\\u003e").replace('&', "\\u0026");
    view! { <leptos_meta::Script type_="application/ld+json">{json}</leptos_meta::Script> }
}
