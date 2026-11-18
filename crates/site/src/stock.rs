//! Firewood availability, one truth for the price list and the e-shop.
//!
//! A firewood section of the price list ("Jasan / Dub") is linked to the
//! e-shop products it stands for (`pricelist_stock_links`). It is available
//! when one of their variants in the matching form is in stock: loose
//! firewood from the container variants, crated firewood from the crates.
//! Setting a section's availability in the admin sets those variants.

#[cfg(feature = "ssr")]
use std::collections::HashMap;

/// The e-shop variant group a firewood price-list category is sold as.
pub fn variant_group(category: &str) -> Option<&'static str> {
    match category {
        "palivove_volne" => Some("kontejner"),
        "palivove_skladane" => Some("bedny"),
        _ => None,
    }
}

/// A price-list category whose sections have an e-shop availability.
pub fn is_firewood_category(category: &str) -> bool {
    variant_group(category).is_some()
}

/// (category, section name) → available, for the sections linked to e-shop
/// variants. A section with no link, or no variants to read, is left out.
#[cfg(feature = "ssr")]
pub async fn pricelist_sections(
    db: &sqlx::SqlitePool,
) -> Result<HashMap<(String, String), bool>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT l.category AS "category!", l.subcategory AS "subcategory!", MAX(v.in_stock) AS "available!: i64"
           FROM pricelist_stock_links l
           JOIN catalog_products p ON p.id = l.catalog_product_id AND p.active = 1
           JOIN catalog_variants v ON v.catalog_product_id = l.catalog_product_id
               AND v.variant_group = CASE l.category WHEN 'palivove_volne' THEN 'kontejner' ELSE 'bedny' END
           GROUP BY l.category, l.subcategory"#
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|r| ((r.category, r.subcategory), r.available != 0))
    .collect())
}
