//! The price list (Ceník) on the contact page and the choices offered in
//! the quote request form (Kalkulace).

use super::tr;
use crate::db::Db;
use crate::locale::Locale;
use crate::money::Money;

pub const FIREWOOD_CATEGORIES: [&str; 2] = ["palivove_volne", "palivove_skladane"];

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct PricelistItem {
    pub id: i64,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub subcategory_de: Option<String>,
    pub item_name: Option<String>,
    pub item_name_de: Option<String>,
    pub details: Option<String>,
    pub details_de: Option<String>,
    pub price: Option<String>,
    pub price_de: Option<String>,
}

impl PricelistItem {
    pub fn item_name(&self, locale: Locale) -> String {
        tr(locale, &self.item_name, &self.item_name_de)
    }
    pub fn details(&self, locale: Locale) -> String {
        tr(locale, &self.details, &self.details_de)
    }
    pub fn subcategory(&self, locale: Locale) -> String {
        tr(locale, &self.subcategory, &self.subcategory_de)
    }

    /// The price as shown: firewood has its own German texts (they are per
    /// m³ in euros already); other prices are converted from crowns.
    pub fn price(&self, money: &Money) -> String {
        let cs_price = self.price.clone().unwrap_or_default();
        if money.locale != Locale::De {
            return cs_price;
        }
        let german = self.price_de.clone().filter(|price| !price.trim().is_empty());
        if FIREWOOD_CATEGORIES.contains(&self.category.as_deref().unwrap_or_default()) {
            german.unwrap_or(cs_price)
        } else {
            money.convert_text(&german.unwrap_or(cs_price))
        }
    }
}

pub async fn pricelist(db: &Db) -> sqlx::Result<Vec<PricelistItem>> {
    sqlx::query_as(
        "SELECT id, category, subcategory, subcategory_de, item_name, item_name_de, details, details_de, price, price_de
         FROM pricelist_items ORDER BY position, id",
    )
    .fetch_all(db)
    .await
}

/// Ceník values are typed in the admin and may contain formatting like
/// m<sup>3</sup>: allow only such tags, never scripts or attributes.
pub fn pricelist_html(text: &str) -> String {
    ammonia::Builder::empty()
        .tags(["sup", "sub", "br", "strong", "em"].into_iter().collect())
        .clean(text)
        .to_string()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pricelist_html_keeps_only_simple_formatting() {
        assert_eq!(pricelist_html("9 800 Kč / m<sup>3</sup>"), "9 800 Kč / m<sup>3</sup>");
        assert_eq!(pricelist_html("<b onclick=\"x()\">a</b><script>alert(1)</script>"), "a");
        assert_eq!(pricelist_html("<strong class=\"x\">b</strong>"), "<strong>b</strong>");
    }
}
