//! schema.org LocalBusiness data (JSON-LD) in the <head> of the home and
//! contact pages, so search engines know the address, hours and phones.

use serde_json::json;

use crate::AppState;
use crate::request::Req;

/// The business data as JSON-LD, in the page's language.
pub fn json(state: &AppState, req: &Req) -> String {
    let t = |key: &str| state.translations.text(req.locale, key, &[]);
    local_business_schema(
        req,
        &t("site.name"),
        &t("meta.home.description"),
        &format!("{}{}", req.base_url, state.assets.url("logo-wm.webp")),
        [t("meta.area_served.south_moravia"), t("meta.area_served.north_austria")],
    )
}

/// schema.org LocalBusiness data (JSON-LD) for the home and contact pages.
/// The facts are the ones in the footer and on the contact page; keep them
/// in sync. No aggregateRating: Google doesn't accept a business's own
/// ratings, and the Google reviews are on Google already.
fn local_business_schema(req: &Req, name: &str, description: &str, logo_url: &str, area: [String; 2]) -> String {
    // The home page: "https://host/" in Czech, "https://host/at" in German.
    let root = format!("{}{}", req.base_url, req.path_for("/"));
    let data = json!({
        "@context": "https://schema.org",
        "@type": "LocalBusiness",
        "@id": format!("{root}#business"),
        "name": name,
        "legalName": "Štěpán Merta",
        "description": description,
        "url": root,
        "logo": logo_url,
        "image": logo_url,
        "telephone": ["+420602446339", "+420515235527"],
        "email": "stepan.merta@seznam.cz",
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
        "areaServed": area,
        "sameAs": ["https://www.facebook.com/piladerflice/"]
    });
    // Nothing in the data may end the <script> tag early.
    data.to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
}
