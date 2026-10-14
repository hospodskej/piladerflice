//! View data shared by the catalogue pages: product cards (e-shop grid,
//! bestsellers, recommendations) and the business facts for search engines.

use std::collections::HashMap;

use serde::Serialize;
use serde_json::json;

use crate::models::catalog::{self, CatalogProduct, CatalogVariant};
use crate::models::uploads::Upload;
use crate::money::Money;
use crate::request::Req;

/// A product tile: picture, name, kind and "from" price.
#[derive(Serialize)]
pub struct ProductCard {
    pub key: String,
    pub title: String,
    pub type_label: String,
    pub image: Option<String>,
    /// "od 1 500 Kč", or None when the product has no variants yet.
    pub price_from: Option<String>,
}

impl ProductCard {
    pub fn new(
        product: &CatalogProduct,
        variants: &[CatalogVariant],
        images: &HashMap<i64, Upload>,
        money: &Money,
        price_from_label: &str,
    ) -> Self {
        let own: Vec<&CatalogVariant> = variants.iter().filter(|v| v.catalog_product_id == product.id).collect();
        Self {
            key: product.key.clone(),
            title: product.title(money.locale),
            type_label: product.type_label(money.locale),
            image: images.get(&product.id).map(Upload::url),
            price_from: catalog::starting_price_czk(&own)
                .map(|czk| format!("{price_from_label} {}", money.display(czk))),
        }
    }
}

/// schema.org LocalBusiness data (JSON-LD) for the home and contact pages.
/// The facts are the ones in the footer and on the contact page; keep them
/// in sync. No aggregateRating: Google doesn't accept a business's own
/// ratings, and the Google reviews are on Google already.
pub fn local_business_schema(req: &Req, name: &str, description: &str, logo_url: &str, area: [String; 2]) -> String {
    // Rails' root_url: "https://host/" in Czech, "https://host/at" in German.
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
    let json = data
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    format!(r#"<script type="application/ld+json">{json}</script>"#)
}

/// Category name for breadcrumbs, by product category.
pub fn category_breadcrumb_key(category: &str) -> &'static str {
    match category {
        "rezivo" => "breadcrumbs.stavebni_rezivo",
        "zbytky" => "breadcrumbs.vyrobni_zbytky",
        "kamenivo" => "breadcrumbs.okrasne_kamenivo",
        _ => "breadcrumbs.palivove_drevo",
    }
}
