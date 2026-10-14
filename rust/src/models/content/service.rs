//! A service (`services`): in the home page accordion and on the Služby page.

use crate::db::Db;
use crate::locale::Locale;
use crate::models::tr;

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
