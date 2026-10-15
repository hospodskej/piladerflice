//! The choices offered in the quote request form (Kalkulace), from
//! `inquiry_form_options`.

use crate::db::Db;
use crate::locale::Locale;
use crate::models::tr;

pub const INQUIRY_CATEGORIES: [&str; 2] = ["palivove", "stavebni"];

/// The columns of an item row in the quote form, per category.
pub fn inquiry_row_fields(category: &str) -> &'static [&'static str] {
    match category {
        "palivove" => &["varianta", "druh", "delka", "mnozstvi"],
        _ => &["polozka", "delka", "vyska", "sirka", "mnozstvi"],
    }
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct InquiryFormOption {
    pub category: String,
    pub field: String,
    pub value: String,
    pub value_de: Option<String>,
}

impl InquiryFormOption {
    pub fn value(&self, locale: Locale) -> String {
        tr(locale, &Some(self.value.clone()), &self.value_de)
    }
}

pub async fn inquiry_options(db: &Db) -> sqlx::Result<Vec<InquiryFormOption>> {
    sqlx::query_as("SELECT category, field, value, value_de FROM inquiry_form_options ORDER BY position, id")
        .fetch_all(db)
        .await
}
