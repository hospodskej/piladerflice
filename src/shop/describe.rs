//! "Smrk, Skládané, 100 cm, 13 PRM": a cart line as text, in a language.
//! The product's current name is used; the
//! specs were copied into the line when it was added.

use std::collections::HashMap;

use super::cart_line::{CartLine, Spec};
use crate::db::Db;
use crate::i18n::Translations;
use crate::locale::Locale;
use crate::views::text::humanize;

/// Product names by key, loaded once per page or e-mail.
pub struct Describer<'a> {
    titles: HashMap<String, (String, Option<String>)>,
    translations: &'a Translations,
}

impl<'a> Describer<'a> {
    pub async fn load(db: &Db, translations: &'a Translations) -> sqlx::Result<Describer<'a>> {
        let rows: Vec<(String, String, Option<String>)> =
            sqlx::query_as("SELECT key, title, title_de FROM catalog_products")
                .fetch_all(db)
                .await?;
        let titles = rows
            .into_iter()
            .map(|(key, title, title_de)| (key, (title, title_de)))
            .collect();
        Ok(Describer { titles, translations })
    }

    pub fn describe(&self, line: &CartLine, locale: Locale) -> String {
        let mut parts = vec![self.title(&line.product_key, locale)];
        parts.extend(line.specs.iter().map(|spec| self.spec_text(spec, locale)));
        parts.join(", ")
    }

    fn title(&self, key: &str, locale: Locale) -> String {
        match self.titles.get(key) {
            Some((title, title_de)) => crate::models::tr(locale, &Some(title.clone()), title_de),
            // A product that is no longer sold (an old order): its old
            // translated title, or else its key made readable.
            None => self
                .translations
                .lookup(locale, &format!("eshop.products.{key}.title"))
                .and_then(|title| title.as_str())
                .map_or_else(|| humanize(key), str::to_string),
        }
    }

    fn spec_text(&self, spec: &Spec, locale: Locale) -> String {
        let value = match &spec.value {
            serde_json::Value::String(text) => text.clone(),
            serde_json::Value::Null => String::new(),
            other => other.to_string(),
        };
        match spec.kind.as_str() {
            "key" => self.translations.text(locale, &value, &[]),
            "amount" => format!(
                "{value} {}",
                self.translations
                    .text(locale, spec.unit_key.as_deref().unwrap_or_default(), &[])
            ),
            "bilingual" => match (&spec.value_de, locale) {
                (Some(german), Locale::De) if !german.trim().is_empty() => german.clone(),
                _ => value,
            },
            _ => value,
        }
    }
}
