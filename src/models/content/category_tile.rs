//! A Sortiment category tile (the `products` table).

use crate::db::Db;
use crate::locale::Locale;
use crate::models::tr;

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
