//! The promo card on the home page (`promos`).

use chrono::{Datelike, NaiveDate};

use crate::db::Db;
use crate::locale::Locale;
use crate::models::tr;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Promo {
    pub id: i64,
    pub title: Option<String>,
    pub title_de: Option<String>,
    pub feature_1: Option<String>,
    pub feature_1_de: Option<String>,
    pub feature_2: Option<String>,
    pub feature_2_de: Option<String>,
    pub feature_3: Option<String>,
    pub feature_3_de: Option<String>,
    pub price: Option<String>,
    pub price_de: Option<String>,
    pub link: Option<String>,
}

impl Promo {
    pub fn title(&self, locale: Locale) -> String {
        tr(locale, &self.title, &self.title_de)
    }
    pub fn features(&self, locale: Locale) -> [String; 3] {
        [
            tr(locale, &self.feature_1, &self.feature_1_de),
            tr(locale, &self.feature_2, &self.feature_2_de),
            tr(locale, &self.feature_3, &self.feature_3_de),
        ]
    }
    /// The price text as typed in the admin; the caller converts crowns to
    /// euros for German visitors.
    pub fn price(&self, locale: Locale) -> String {
        tr(locale, &self.price, &self.price_de)
    }
}

/// The promo on the home page: they take turns, one per calendar week.
pub async fn weekly_promo(db: &Db, today: NaiveDate) -> sqlx::Result<Option<Promo>> {
    let mut promos: Vec<Promo> = sqlx::query_as(
        "SELECT id, title, title_de, feature_1, feature_1_de, feature_2, feature_2_de, feature_3, feature_3_de,
                price, price_de, link FROM promos ORDER BY id",
    )
    .fetch_all(db)
    .await?;
    if promos.is_empty() {
        return Ok(None);
    }
    let index = today.iso_week().week() as usize % promos.len();
    Ok(Some(promos.swap_remove(index)))
}
