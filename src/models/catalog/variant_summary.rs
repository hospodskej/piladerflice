//! What product lists need from the variants, worked out by the database:
//! one row per product instead of every variant (the e-shop grid, the home
//! page bestsellers and the recommendations only show a "from" price).

use std::collections::HashMap;

use chrono::NaiveDateTime;

use crate::db::Db;

/// Each product's "from" price: its cheapest variant in stock, or the
/// cheapest of all when none is in stock. Products without variants are
/// missing.
pub async fn starting_prices(db: &Db) -> sqlx::Result<HashMap<i64, i64>> {
    let rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT catalog_product_id,
           COALESCE(MIN(CASE WHEN in_stock THEN price_czk END), MIN(price_czk))
         FROM catalog_variants GROUP BY catalog_product_id",
    )
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().collect())
}

/// When each product's newest variant was last changed (the sitemap's
/// `lastmod`). Products without variants are missing.
pub async fn last_changes(db: &Db) -> sqlx::Result<HashMap<i64, NaiveDateTime>> {
    let rows: Vec<(i64, NaiveDateTime)> =
        sqlx::query_as("SELECT catalog_product_id, MAX(updated_at) FROM catalog_variants GROUP BY catalog_product_id")
            .fetch_all(db)
            .await?;
    Ok(rows.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn database_with(variants: &[(i64, i64, bool, &str)]) -> Db {
        let db = crate::db::in_memory().await.unwrap();
        for product in [1, 2, 3] {
            sqlx::query(
                "INSERT INTO catalog_products (id, category, key, template, title, created_at, updated_at)
                 VALUES (?, 'palivove', ?, 'firewood', 'Test', '2026-01-01', '2026-01-01')",
            )
            .bind(product)
            .bind(format!("product-{product}"))
            .execute(&db)
            .await
            .unwrap();
        }
        for (n, (product, price, in_stock, updated_at)) in variants.iter().enumerate() {
            sqlx::query(
                "INSERT INTO catalog_variants (catalog_product_id, key, price_czk, in_stock, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(product)
            .bind(format!("variant-{n}"))
            .bind(price)
            .bind(in_stock)
            .bind(updated_at)
            .bind(updated_at)
            .execute(&db)
            .await
            .unwrap();
        }
        db
    }

    #[tokio::test]
    async fn starting_price_prefers_variants_in_stock() {
        let db = database_with(&[
            // 1: the cheapest is sold out, so the cheapest in stock.
            (1, 100, false, "2026-01-01 10:00:00"),
            (1, 300, true, "2026-01-01 10:00:00"),
            (1, 200, true, "2026-01-01 10:00:00"),
            // 2: nothing in stock, so the cheapest of all.
            (2, 500, false, "2026-01-01 10:00:00"),
            (2, 400, false, "2026-01-01 10:00:00"),
            // 3 has no variants.
        ])
        .await;
        let prices = starting_prices(&db).await.unwrap();
        assert_eq!(prices, HashMap::from([(1, 200), (2, 400)]));
    }

    #[tokio::test]
    async fn last_change_is_the_newest_variant() {
        let db = database_with(&[
            (1, 100, true, "2026-03-01 08:00:00.123456"),
            (1, 100, true, "2026-05-01 08:00:00.5"),
            (2, 100, true, "2026-02-01 08:00:00"),
        ])
        .await;
        let changes = last_changes(&db).await.unwrap();
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[&1].to_string(), "2026-05-01 08:00:00.500");
        assert_eq!(changes[&2].to_string(), "2026-02-01 08:00:00");
    }
}
