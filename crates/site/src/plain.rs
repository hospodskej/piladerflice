//! Responses that aren't pages: robots.txt, sitemap.xml, llms.txt, the
//! health check, and redirects from the old site's addresses.

use crate::i18n::Locale;
use crate::state::AppState;
use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use std::fmt::Write as _;

/// "https://piladerflice.cz": APP_HOST in production, else the request's host.
fn base_url(headers: &HeaderMap) -> String {
    match std::env::var("APP_HOST") {
        Ok(host) if !host.is_empty() => format!("https://{host}"),
        _ => format!(
            "http://{}",
            headers
                .get(header::HOST)
                .and_then(|h| h.to_str().ok())
                .unwrap_or("localhost")
        ),
    }
}

/// Cached by browsers and proxies for an hour.
fn hourly(content_type: &'static str, body: String) -> Response {
    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "max-age=3600, public"),
        ],
        body,
    )
        .into_response()
}

async fn robots(headers: HeaderMap) -> Response {
    // Private or per-visitor pages have nothing worth finding in a search
    // engine. Keep the rules directly under User-agent: some parsers end the
    // group at a blank line.
    hourly(
        "text/plain; charset=utf-8",
        format!(
            "User-agent: *\nDisallow: /admin\nDisallow: /kosik\nDisallow: /muj-ucet\nDisallow: /prihlaseni\nDisallow: /registrace\n\
             Disallow: /at/kosik\nDisallow: /at/muj-ucet\nDisallow: /at/prihlaseni\nDisallow: /at/registrace\n\nSitemap: {}/sitemap.xml\n",
            base_url(&headers)
        ),
    )
}

const STATIC_PAGES: [&str; 12] = [
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

async fn sitemap(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let base = base_url(&headers);
    let mut entries: Vec<(String, Option<String>)> = STATIC_PAGES
        .iter()
        .map(|p| ((*p).to_owned(), None))
        .collect();
    entries.push(("/eshop".into(), None));
    // Firewood is the shop's first tab, so its category address isn't listed.
    let categories =
        sqlx::query_scalar!("SELECT DISTINCT category FROM catalog_products WHERE active = 1")
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();
    for category in domain::Category::ALL
        .iter()
        .map(|c| c.as_str())
        .filter(|c| *c != "palivove" && categories.iter().any(|x| x == c))
    {
        entries.push((format!("/eshop?category={category}"), None));
    }
    let products = sqlx::query!(
        r#"SELECT p.key, MAX(p.updated_at, COALESCE((SELECT MAX(v.updated_at) FROM catalog_variants v WHERE v.catalog_product_id = p.id), '')) AS "lastmod!: String"
           FROM catalog_products p WHERE p.active = 1 ORDER BY p.position, p.id"#
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    entries.extend(products.into_iter().map(|p| {
        (
            format!("/eshop/{}", p.key),
            Some(p.lastmod.chars().take(19).collect::<String>() + "Z"),
        )
    }));

    let escape = |s: &str| {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('"', "&quot;")
    };
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n",
    );
    for (path, lastmod) in entries {
        let cs = escape(&format!("{base}{}", Locale::Cs.path(&path)));
        let de = escape(&format!("{base}{}", Locale::De.path(&path)));
        for loc in [&cs, &de] {
            let _ = write!(xml, "  <url>\n    <loc>{loc}</loc>\n");
            if let Some(lastmod) = &lastmod {
                let _ = writeln!(xml, "    <lastmod>{lastmod}</lastmod>");
            }
            let _ = write!(
                xml,
                "    <xhtml:link rel=\"alternate\" hreflang=\"cs\" href=\"{cs}\"/>\n    <xhtml:link rel=\"alternate\" hreflang=\"de-AT\" href=\"{de}\"/>\n    <xhtml:link rel=\"alternate\" hreflang=\"x-default\" href=\"{cs}\"/>\n  </url>\n"
            );
        }
    }
    xml.push_str("</urlset>\n");
    hourly("application/xml; charset=utf-8", xml)
}

/// A summary of the business and the catalogue for AI assistants.
async fn llms(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let base = base_url(&headers);
    let per_km =
        sqlx::query_scalar!("SELECT value FROM settings WHERE key = 'delivery_czk_per_km'")
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| domain::delivery::DEFAULT_CZK_PER_KM.to_string());
    let rules = crate::shop::load(&state)
        .await
        .map(|s| s.vat)
        .unwrap_or_default();
    let products = sqlx::query!(
        r#"SELECT p.id AS "id!", p.key, p.title, p.type_label, p.category, p.template FROM catalog_products p WHERE p.active = 1 ORDER BY p.position, p.id"#
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    let variants = sqlx::query!(
        "SELECT catalog_product_id, price_czk, in_stock, amount_value FROM catalog_variants"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut text = format!(
        "# Pila Derflice\n\n\
> Family-run sawmill in Derflice near Znojmo (South Moravia, Czech Republic). Sells firewood, construction and joinery lumber, sawmill by-products (sawdust, wood chips, slabs) and decorative stone, and does custom sawing, planing and impregnation. Delivers with its own truck across South Moravia and northern Austria; pickup at the sawmill is free. Orders through the online shop or an inquiry form; the site is in Czech, with a German version for Austria under /at.\n\n\
Key facts:\n\n\
- Address: Derflice 66, 671 28 Znojmo, Czech Republic (business owner Štěpán Merta, IČ 45665451, DIČ CZ6909094742)\n\
- Phone: +420 602 446 339, +420 515 235 527; e-mail: stepan.merta@seznam.cz\n\
- Opening hours: Monday to Friday 7:00–15:30, other times by arrangement\n\
- Prices are in Czech crowns (CZK) and include VAT. The German site shows them converted to euros.\n\
- Delivery to an address costs {per_km} Kč per km, counted there and back, one trip per order; the shop calculates it from the delivery address (customers can also type a town or postcode in the e-shop to see it before ordering). Pickup at the sawmill is free.\n\
- Payment: cash on pickup, or bank transfer against a proforma invoice.\n\
- For quantities, sizes or products not in the shop, customers send an inquiry from the contact page and get a quote.\n\n\
## Online shop\n\n"
    );
    let names = [
        ("palivove", "Firewood (palivové dřevo)"),
        ("rezivo", "Construction lumber (stavební řezivo)"),
        ("zbytky", "Sawmill by-products (výrobní zbytky)"),
        ("kamenivo", "Decorative stone (okrasné kamenivo)"),
    ];
    for (category, name) in names {
        let in_category: Vec<_> = products.iter().filter(|p| p.category == category).collect();
        if in_category.is_empty() {
            continue;
        }
        let _ = write!(text, "### {name}\n\n");
        for p in in_category {
            let own: Vec<_> = variants
                .iter()
                .filter(|v| v.catalog_product_id == p.id)
                .collect();
            let in_stock: Vec<_> = own.iter().filter(|v| v.in_stock != 0).copied().collect();
            let candidates = if in_stock.is_empty() { own } else { in_stock };
            let note = candidates.into_iter().min_by_key(|v| v.price_czk).and_then(|cheapest| {
                let category = p.category.parse().ok()?;
                let price = rules.customer_price(category, domain::Czk(cheapest.price_czk));
                match p.template.as_str() {
                    "firewood" => cheapest.amount_value.map(|amount| format!("from {price} for {amount} PRM (prostorový metr, a cubic metre of piled wood)")),
                    "lumber" => Some(format!("from {price} per piece")),
                    _ => None,
                }
            });
            let _ = writeln!(
                text,
                "- [{}]({base}/eshop/{}): {}{}",
                p.title,
                p.key,
                p.type_label.as_deref().unwrap_or_default(),
                note.map(|n| format!("; {n}")).unwrap_or_default()
            );
        }
        text.push('\n');
    }
    let _ = write!(
        text,
        "Prices include VAT. Product pages list every size, length and amount with its price and an add-to-cart button; the contact page's price list has the rest (by-products, stone, services).\n\n\
## Pages\n\n\
- [Contact, price list and inquiry form]({base}/kontakt): phone, e-mail, map, the full price list (lumber, firewood, by-products, services, delivery) and the form for a free quote\n\
- [Online shop]({base}/eshop): all products by category\n\
- [Services]({base}/sluzby): custom sawing, planing, impregnation, delivery\n\
- [Range overview]({base}/sortiment): firewood, construction and joinery lumber, by-products, decorative stone\n\
- [Terms and conditions]({base}/obchodni-podminky): ordering, delivery, payment, returns\n\n\
## German version (for Austria)\n\n\
- [Startseite]({base}/at)\n\
- [Onlineshop]({base}/at/eshop)\n\
- [Kontakt, Preisliste und Anfrage]({base}/at/kontakt)\n\n\
## Optional\n\n\
- [Privacy policy]({base}/zasady-ochrany-osobnich-udaju)\n\
- [Sitemap]({base}/sitemap.xml): every page in Czech and German\n"
    );
    hourly("text/plain; charset=utf-8", text)
}

/// 200 when the app is up and the database answers (for the proxy's health checks).
async fn up(State(state): State<AppState>) -> StatusCode {
    match sqlx::query_scalar!("SELECT 1 AS \"one!: i64\"")
        .fetch_one(&state.db)
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Addresses of the old WordPress site (until the 2026 launch), from its
/// sitemaps. Search engines and bookmarks still know them, so each sends a
/// permanent redirect to the page that replaced it.
pub const OLD_ADDRESSES: [(&str, &str); 37] = [
    ("/obchod", "/eshop"),
    ("/obchod/palivove-drevo", "/eshop?category=palivove"),
    ("/obchod/stavebni-rezivo", "/eshop?category=rezivo"),
    ("/obchod/vyrobni-zbytky", "/eshop?category=zbytky"),
    ("/obchod/okrasne-kamenivo", "/eshop?category=kamenivo"),
    ("/pokladna", "/kosik"),
    ("/zakazkova-vyroba", "/kontakt#kalkulace"),
    ("/fotogalerie", "/sortiment"),
    ("/zasady-cookies-eu", "/zasady-cookies"),
    ("/produkt/palivove-drevo-mekke-smrk", "/eshop/smrk"),
    ("/produkt/palivove-drevo-tvrde-akat", "/eshop/akat"),
    ("/produkt/palivove-drevo-tvrde-briza", "/eshop/briza"),
    ("/produkt/palivove-drevo-tvrde-buk", "/eshop/buk"),
    ("/produkt/palivove-drevo-tvrde-dub", "/eshop/dub"),
    ("/produkt/palivove-drevo-tvrde-habr", "/eshop/habr"),
    ("/produkt/palivove-drevo-tvrde-jasan", "/eshop/jasan"),
    ("/produkt/hranoly", "/eshop/tramy"),
    ("/produkt/fosny", "/eshop/fosny"),
    ("/produkt/prkna-24-mm", "/eshop/prkna"),
    ("/produkt/stresni-late", "/eshop/late"),
    ("/produkt/odkory-na-topeni", "/eshop/odkory"),
    ("/produkt/piliny", "/eshop/piliny"),
    ("/produkt/stepka", "/eshop/stepka"),
    ("/produkt/okrasne-kamenivo", "/eshop/okrasne_kamenivo"),
    // The German site was under /de; it's /at now.
    ("/de", "/at"),
    ("/de/sortiment", "/at/sortiment"),
    ("/de/sortiment/brennholz", "/at/sortiment/palivove-drevo"),
    ("/de/sortiment/bauholz", "/at/sortiment/stavebni-rezivo"),
    (
        "/de/sortiment/tischlerholz",
        "/at/sortiment/truhlarske-rezivo",
    ),
    (
        "/de/sortiment/produktionsreste",
        "/at/sortiment/vyrobni-zbytky",
    ),
    (
        "/de/sortiment/ornamentale-aggregate",
        "/at/sortiment/okrasne-kamenivo",
    ),
    ("/de/eshop/brennholz-eshop", "/at/eshop?category=palivove"),
    ("/de/eshop/bauholz-eshop", "/at/eshop?category=rezivo"),
    (
        "/de/eshop/produktionsreste-eshop",
        "/at/eshop?category=zbytky",
    ),
    ("/de/eshop/ziersteine-eshop", "/at/eshop?category=kamenivo"),
    ("/de/massanfertigung", "/at/kontakt#kalkulace"),
    ("/de/dienstleistungen", "/at/sluzby"),
];

/// More of the old German site, sent to the closest page.
const OLD_GERMAN_ADDRESSES: [(&str, &str); 4] = [
    ("/de/fotogalerie", "/at/sortiment"),
    ("/de/kontakt", "/at/kontakt"),
    ("/de/warenkorb", "/at/kosik"),
    ("/de/zur-kasse", "/at/kosik"),
];

/// The redirect for an old address, if `path` is one: the known ones first,
/// then anything else under /produkt, /obchod or /de.
pub fn old_address(path: &str) -> Option<&'static str> {
    if let Some((_, new)) = OLD_ADDRESSES
        .iter()
        .chain(OLD_GERMAN_ADDRESSES.iter())
        .find(|(old, _)| *old == path)
    {
        return Some(new);
    }
    if path.starts_with("/produkt/") || path.starts_with("/obchod/") {
        Some("/eshop")
    } else if path.starts_with("/de/") {
        Some("/at")
    } else {
        None
    }
}

/// A 301 (Moved Permanently), as the old site's redirects were.
fn moved(location: &str) -> Response {
    (
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, location.to_owned())],
    )
        .into_response()
}

/// Middleware: www.<APP_HOST> → <APP_HOST> (same path), old addresses → new
/// ones, ?locale=de links → /at; all permanent.
pub async fn redirects(request: Request, next: Next) -> Response {
    let uri = request.uri();
    if let Ok(host) = std::env::var("APP_HOST") {
        let asked = request
            .headers()
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default();
        if !host.is_empty() && asked == format!("www.{host}") {
            let path = uri.path_and_query().map_or("/", |p| p.as_str());
            return moved(&format!("https://{host}{path}"));
        }
    }
    // "/eshop/" is "/eshop", for old addresses too (WordPress ended them in
    // "/"). Never "//host/": that would leave the site.
    let path = uri.path();
    let trimmed = path.trim_end_matches('/');
    if let Some(new) = old_address(if trimmed.is_empty() { "/" } else { trimmed }) {
        return moved(new);
    }
    if path.len() > 1
        && trimmed.len() < path.len()
        && !trimmed.starts_with("//")
        && matches!(
            *request.method(),
            axum::http::Method::GET | axum::http::Method::HEAD
        )
    {
        let trimmed = if trimmed.is_empty() { "/" } else { trimmed };
        return match uri.query() {
            Some(query) => moved(&format!("{trimmed}?{query}")),
            None => moved(trimmed),
        };
    }
    // Links from before the /at prefix: ?locale=de on an unprefixed address.
    let is_get = matches!(
        *request.method(),
        axum::http::Method::GET | axum::http::Method::HEAD
    );
    if let (true, Some(query)) = (is_get, uri.query()) {
        let (locale, rest): (Vec<&str>, Vec<&str>) =
            query.split('&').partition(|p| p.starts_with("locale="));
        let target = locale
            .first()
            .and_then(|p| p.strip_prefix("locale="))
            .and_then(|l| l.parse::<Locale>().ok());
        if let (Some(target), Locale::Cs) = (target, Locale::of_path(uri.path())) {
            let path = target.path(uri.path());
            let location = if rest.is_empty() {
                path
            } else {
                format!("{path}?{}", rest.join("&"))
            };
            return moved(&location);
        }
    }
    next.run(request).await
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/robots.txt", get(robots))
        .route("/sitemap.xml", get(sitemap))
        .route("/llms.txt", get(llms))
        .route("/up", get(up))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_addresses() {
        assert_eq!(old_address("/produkt/hranoly"), Some("/eshop/tramy"));
        assert_eq!(old_address("/produkt/neco-jineho"), Some("/eshop"));
        assert_eq!(old_address("/de/kontakt"), Some("/at/kontakt"));
        assert_eq!(old_address("/de/irgendwas"), Some("/at"));
        assert_eq!(old_address("/kontakt"), None);
        assert_eq!(old_address("/obchod"), Some("/eshop"));
    }
}
