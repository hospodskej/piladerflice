//! A slide of the "Co by vás mohlo zajímat" carousel (`faq_items`).

use crate::db::Db;
use crate::locale::Locale;
use crate::models::tr;

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
