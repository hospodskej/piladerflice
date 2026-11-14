//! Catalogue queries shared by several pages.

#[cfg(feature = "ssr")]
use crate::pages::home::Bestseller;

/// Active products, best-selling first (quantities sold in the last three
/// months), then in the `fallback` order, then by position.
#[cfg(feature = "ssr")]
pub async fn bestsellers(
    db: &sqlx::SqlitePool,
    v: &crate::visitor::Visitor,
    fallback: &[&str],
) -> Result<Vec<Bestseller>, sqlx::Error> {
    use crate::shop::translated as tr;
    use domain::Category;
    use domain::money::Czk;
    use std::collections::HashMap;

    let since = (time::OffsetDateTime::now_utc() - time::Duration::days(91))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    let sold: HashMap<String, i64> = sqlx::query!(
        r#"SELECT json_extract(item.value, '$.product_key') AS "key: String",
                  SUM(json_extract(item.value, '$.quantity')) AS "sold: i64"
           FROM orders, json_each(orders.items) AS item
           WHERE orders.created_at >= ?
           GROUP BY 1"#,
        since
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .filter_map(|r| Some((r.key?, r.sold.unwrap_or(0))))
    .collect();

    let mut products = sqlx::query!(
        r#"SELECT p.id AS "id!", p.key, p.title, p.title_de, p.type_label, p.type_label_de, p.image, p.category, p.position, p.hardness,
                  (SELECT COALESCE(MIN(CASE WHEN v.in_stock THEN v.price_czk END), MIN(v.price_czk))
                   FROM catalog_variants v WHERE v.catalog_product_id = p.id) AS "starting_price: i64"
           FROM catalog_products p WHERE p.active = 1"#
    )
    .fetch_all(db)
    .await?;
    products.sort_by_key(|p| {
        (
            -sold.get(&p.key).copied().unwrap_or(0),
            fallback
                .iter()
                .position(|k| *k == p.key)
                .unwrap_or(fallback.len()),
            p.position,
            p.id,
        )
    });

    let l = v.locale;
    Ok(products
        .into_iter()
        .map(|p| {
            let price_from = p
                .starting_price
                .zip(p.category.parse::<Category>().ok())
                .map(|(price, category)| {
                    v.shop
                        .price(l, v.shop.vat.customer_price(category, Czk(price)))
                });
            Bestseller {
                id: p.id,
                category: p.category.clone(),
                hardness: p.hardness.clone(),
                position: p.position,
                title: tr(l, &p.title, p.title_de.as_deref()).to_owned(),
                type_label: tr(
                    l,
                    p.type_label.as_deref().unwrap_or_default(),
                    p.type_label_de.as_deref(),
                )
                .to_owned(),
                image: p.image,
                price_from,
                key: p.key,
            }
        })
        .collect())
}
