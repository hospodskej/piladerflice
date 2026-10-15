//! /sitemap.xml: every public page in Czech and German, linked together, so
//! search engines find both versions.

use axum::extract::State;
use axum::http::header;
use axum::response::IntoResponse;
use chrono::{NaiveDateTime, SecondsFormat};

use super::Result;
use crate::AppState;
use crate::locale::{Locale, localized_path};
use crate::models::catalog::{self, CATEGORIES};
use crate::request::Req;
use crate::views::escape;

const STATIC_PATHS: [&str; 12] = [
    "/",
    "/kontakt",
    "/sluzby",
    "/sortiment",
    "/sortiment/palivove-drevo",
    "/sortiment/stavebni-rezivo",
    "/sortiment/truhlarske-rezivo",
    "/sortiment/okrasne-kamenivo",
    "/sortiment/vyrobni-zbytky",
    "/obchodni-podminky",
    "/zasady-cookies",
    "/zasady-ochrany-osobnich-udaju",
];

pub async fn show(State(state): State<AppState>, req: Req) -> Result {
    let products = catalog::active_products(&state.db).await?;
    let variants = catalog::all_variants(&state.db).await?;

    let mut entries: Vec<(String, Option<NaiveDateTime>)> =
        STATIC_PATHS.iter().map(|path| (path.to_string(), None)).collect();
    // /eshop already is the firewood listing; other categories get their own
    // entry when they have something to show.
    entries.push(("/eshop".into(), None));
    for category in CATEGORIES.iter().filter(|c| **c != "palivove") {
        if products.iter().any(|product| product.category == *category) {
            entries.push((format!("/eshop?category={category}"), None));
        }
    }
    for product in &products {
        let lastmod = variants
            .iter()
            .filter(|variant| variant.catalog_product_id == product.id)
            .map(|variant| variant.updated_at)
            .chain([product.updated_at])
            .max();
        entries.push((format!("/eshop/{}", product.key), lastmod));
    }

    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n",
    );
    for (path, lastmod) in &entries {
        let urls: Vec<(&str, String)> = Locale::ALL
            .iter()
            .map(|locale| {
                (
                    locale.hreflang(),
                    format!("{}{}", req.base_url, localized_path(path, *locale)),
                )
            })
            .collect();
        for (_, loc) in &urls {
            xml.push_str("  <url>\n");
            xml.push_str(&format!("    <loc>{}</loc>\n", escape(loc)));
            if let Some(lastmod) = lastmod {
                xml.push_str(&format!(
                    "    <lastmod>{}</lastmod>\n",
                    lastmod.and_utc().to_rfc3339_opts(SecondsFormat::Secs, true)
                ));
            }
            for (hreflang, href) in &urls {
                xml.push_str(&format!(
                    "    <xhtml:link rel=\"alternate\" hreflang=\"{hreflang}\" href=\"{}\"/>\n",
                    escape(href)
                ));
            }
            xml.push_str(&format!(
                "    <xhtml:link rel=\"alternate\" hreflang=\"x-default\" href=\"{}\"/>\n",
                escape(&urls[0].1)
            ));
            xml.push_str("  </url>\n");
        }
    }
    xml.push_str("</urlset>\n");

    Ok((
        [
            (header::CONTENT_TYPE, "application/xml; charset=utf-8"),
            (header::CACHE_CONTROL, "max-age=3600, public"),
        ],
        xml,
    )
        .into_response())
}
