//! Content edited in the admin or seeded: the weekly promo, services,
//! Sortiment categories, the "might interest you" slides and Google reviews.

use chrono::{Datelike, NaiveDate};

use super::tr;
use crate::db::Db;
use crate::locale::Locale;

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

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Service {
    pub id: i64,
    pub title: Option<String>,
    pub title_de: Option<String>,
    pub content: Option<String>,
    pub content_de: Option<String>,
    pub button_text: Option<String>,
    pub button_text_de: Option<String>,
    pub button_path: Option<String>,
    /// A JSON list of image paths, e.g. ["sluzby/doprava1.webp"].
    pub images: Option<String>,
}

impl Service {
    pub fn title(&self, locale: Locale) -> String {
        tr(locale, &self.title, &self.title_de)
    }
    pub fn content(&self, locale: Locale) -> String {
        tr(locale, &self.content, &self.content_de)
    }
    pub fn button_text(&self, locale: Locale) -> String {
        tr(locale, &self.button_text, &self.button_text_de)
    }
    pub fn image_list(&self) -> Vec<String> {
        self.images
            .as_deref()
            .and_then(|json| serde_json::from_str(json).ok())
            .unwrap_or_default()
    }
}

const SERVICE_COLUMNS: &str =
    "id, title, title_de, content, content_de, button_text, button_text_de, button_path, images";

pub async fn services(db: &Db) -> sqlx::Result<Vec<Service>> {
    sqlx::query_as(&format!("SELECT {SERVICE_COLUMNS} FROM services ORDER BY id"))
        .fetch_all(db)
        .await
}

/// The services that have a section on the Služby page.
pub async fn services_page(db: &Db) -> sqlx::Result<Vec<Service>> {
    sqlx::query_as(&format!(
        "SELECT {SERVICE_COLUMNS} FROM services WHERE button_path LIKE '/sluzby%' ORDER BY id"
    ))
    .fetch_all(db)
    .await
}

/// A Sortiment category tile (the `products` table): image, title, text, link.
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct CategoryTile {
    pub id: i64,
    pub title: Option<String>,
    pub title_de: Option<String>,
    pub text: Option<String>,
    pub text_de: Option<String>,
    pub image: Option<String>,
    pub link: Option<String>,
}

impl CategoryTile {
    pub fn title(&self, locale: Locale) -> String {
        tr(locale, &self.title, &self.title_de)
    }
    pub fn text(&self, locale: Locale) -> String {
        tr(locale, &self.text, &self.text_de)
    }
}

pub async fn category_tiles(db: &Db) -> sqlx::Result<Vec<CategoryTile>> {
    sqlx::query_as("SELECT id, title, title_de, text, text_de, image, link FROM products ORDER BY id")
        .fetch_all(db)
        .await
}

/// A slide of the "Co by vás mohlo zajímat" carousel.
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct FaqItem {
    pub id: i64,
    pub title: Option<String>,
    pub title_de: Option<String>,
    pub content: Option<String>,
    pub content_de: Option<String>,
    pub image: Option<String>,
}

impl FaqItem {
    pub fn title(&self, locale: Locale) -> String {
        tr(locale, &self.title, &self.title_de)
    }
    pub fn content(&self, locale: Locale) -> String {
        tr(locale, &self.content, &self.content_de)
    }
}

pub async fn faq_items(db: &Db) -> sqlx::Result<Vec<FaqItem>> {
    sqlx::query_as("SELECT id, title, title_de, content, content_de, image FROM faq_items ORDER BY id")
        .fetch_all(db)
        .await
}

/// A Google review, copied into the site.
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Review {
    pub id: i64,
    pub name: String,
    pub avatar: Option<String>,
    pub stars: i64,
    pub text: String,
    pub reviewed_on: NaiveDate,
}

const AVATAR_COLORS: [&str; 8] = [
    "#4285F4", "#EA4335", "#9C27B0", "#0F9D58", "#F4B400", "#FF7043", "#00ACC1", "#8E24AA",
];

impl Review {
    pub fn initial(&self) -> String {
        self.name
            .chars()
            .next()
            .map(|c| c.to_uppercase().collect())
            .unwrap_or_default()
    }

    pub fn avatar_color(&self) -> &'static str {
        AVATAR_COLORS[self.id.rem_euclid(AVATAR_COLORS.len() as i64) as usize]
    }

    /// "před 2 lety" / "vor 3 Monaten": how long ago, in whole years or months.
    pub fn time_ago(&self, locale: Locale, today: NaiveDate) -> String {
        let then = self.reviewed_on;
        let mut years = today.year() - then.year();
        if (today.month(), today.day()) < (then.month(), then.day()) {
            years -= 1;
        }
        let (count, cs_one, cs_many, de_one, de_many) = if years >= 1 {
            (years, "rokem", "lety", "Jahr", "Jahren")
        } else {
            let mut months = (today.year() - then.year()) * 12 + today.month() as i32 - then.month() as i32;
            if today.day() < then.day() {
                months -= 1;
            }
            (months.max(1), "měsícem", "měsíci", "Monat", "Monaten")
        };
        match (locale, count) {
            (Locale::De, 1) => format!("vor 1 {de_one}"),
            (Locale::De, n) => format!("vor {n} {de_many}"),
            (Locale::Cs, 1) => format!("před {cs_one}"),
            (Locale::Cs, n) => format!("před {n} {cs_many}"),
        }
    }
}

pub async fn reviews(db: &Db) -> sqlx::Result<Vec<Review>> {
    sqlx::query_as("SELECT id, name, avatar, stars, text, reviewed_on FROM reviews ORDER BY position, id")
        .fetch_all(db)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn review(on: &str) -> Review {
        Review {
            id: 3,
            name: "šimon".into(),
            avatar: None,
            stars: 5,
            text: String::new(),
            reviewed_on: NaiveDate::parse_from_str(on, "%Y-%m-%d").unwrap(),
        }
    }

    #[test]
    fn time_ago_in_years_or_months() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 14).unwrap();
        assert_eq!(review("2024-10-14").time_ago(Locale::Cs, today), "před 2 lety");
        assert_eq!(review("2025-10-15").time_ago(Locale::Cs, today), "před 11 měsíci");
        assert_eq!(review("2025-10-14").time_ago(Locale::De, today), "vor 1 Jahr");
        assert_eq!(review("2026-10-01").time_ago(Locale::Cs, today), "před měsícem");
        assert_eq!(review("2026-08-14").time_ago(Locale::De, today), "vor 2 Monaten");
    }

    #[test]
    fn avatar_initial_and_color() {
        assert_eq!(review("2026-01-01").initial(), "Š");
        assert_eq!(review("2026-01-01").avatar_color(), "#0F9D58");
    }
}
