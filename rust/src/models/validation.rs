//! Validation errors with Rails' wording: "Klíč už je použitý" is the
//! field's name plus the message, both from the translations, looked up the
//! way Rails looks them up (most specific first):
//!
//! 1. `activerecord.errors.models.<model>.attributes.<field>.<error>`
//! 2. `activerecord.errors.models.<model>.<error>`
//! 3. `activerecord.errors.messages.<error>`
//! 4. `errors.attributes.<field>.<error>`
//! 5. `errors.messages.<error>`
//!
//! The field's name is `activerecord.attributes.<model>.<field>`, or the
//! column name made readable ("price_czk" -> "Price czk") like Rails does.

use serde::Serialize;

use crate::i18n::{Translations, interpolate};
use crate::locale::Locale;
use crate::views::text::humanize;

/// The errors of one record, in the order they were found.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Errors {
    /// (field, full message)
    list: Vec<(String, String)>,
}

impl Errors {
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn messages(&self) -> Vec<String> {
        self.list.iter().map(|(_, message)| message.clone()).collect()
    }

    /// The fields that have an error (their inputs are marked in the form).
    pub fn fields(&self) -> Vec<String> {
        self.list.iter().map(|(field, _)| field.clone()).collect()
    }

    pub fn push(&mut self, field: &str, message: String) {
        self.list.push((field.to_string(), message));
    }
}

/// Builds the messages of one kind of record ("catalog_product").
pub struct Validator<'a> {
    translations: &'a Translations,
    locale: Locale,
    model: &'a str,
    pub errors: Errors,
}

impl<'a> Validator<'a> {
    pub fn new(translations: &'a Translations, locale: Locale, model: &'a str) -> Self {
        Self {
            translations,
            locale,
            model,
            errors: Errors::default(),
        }
    }

    /// Adds `error` ("blank", "taken"...) for `field`.
    pub fn add(&mut self, field: &str, error: &str) {
        self.add_with_count(field, error, None);
    }

    pub fn add_with_count(&mut self, field: &str, error: &str, count: Option<i64>) {
        let message = full_message(self.translations, self.locale, self.model, field, error, count);
        self.errors.push(field, message);
    }

    pub fn finish(self) -> Errors {
        self.errors
    }
}

/// "Klíč už je použitý".
pub fn full_message(
    translations: &Translations,
    locale: Locale,
    model: &str,
    field: &str,
    error: &str,
    count: Option<i64>,
) -> String {
    let keys = [
        format!("activerecord.errors.models.{model}.attributes.{field}.{error}"),
        format!("activerecord.errors.models.{model}.{error}"),
        format!("activerecord.errors.messages.{error}"),
        format!("errors.attributes.{field}.{error}"),
        format!("errors.messages.{error}"),
    ];
    let count = count.map(|count| count.to_string()).unwrap_or_default();
    let message = keys
        .iter()
        .find_map(|key| translations.lookup(locale, key).and_then(|text| text.as_str()))
        .map(|text| interpolate(text, &[("count", &count)]))
        .unwrap_or_else(|| error.replace('_', " "));
    format!("{} {message}", attribute_name(translations, locale, model, field))
}

/// "Klíč", or "Price czk" when there is no translation.
pub fn attribute_name(translations: &Translations, locale: Locale, model: &str, field: &str) -> String {
    let key = format!("activerecord.attributes.{model}.{field}");
    match translations.lookup(locale, &key).and_then(|text| text.as_str()) {
        Some(name) => name.to_string(),
        None => humanize(field),
    }
}

/// "A, B a C": messages joined into one sentence (Rails' `to_sentence`),
/// with the connectors from `support.array` in the translations.
pub fn to_sentence(translations: &Translations, locale: Locale, items: &[String]) -> String {
    let connector = |name: &str, default: &str| {
        translations
            .lookup(locale, &format!("support.array.{name}"))
            .and_then(|text| text.as_str())
            .unwrap_or(default)
            .to_string()
    };
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [first, second] => format!("{first}{}{second}", connector("two_words_connector", " and ")),
        [rest @ .., last] => format!(
            "{}{}{last}",
            rest.join(&connector("words_connector", ", ")),
            connector("last_word_connector", ", and ")
        ),
    }
}

/// "1 chyba", "3 chyby", "5 chyb" (Czech plural).
pub fn error_count(count: usize) -> String {
    let word = match count {
        1 => "chyba",
        2..=4 => "chyby",
        _ => "chyb",
    };
    format!("{count} {word}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn translations() -> Translations {
        Translations::load(&crate::Config::from_source_tree(false).app_root.join("config/locales")).unwrap()
    }

    #[test]
    fn messages_come_from_the_most_specific_translation() {
        let t = translations();
        // Specific to the model and field.
        assert_eq!(
            full_message(&t, Locale::Cs, "pricelist_item", "price", "blank", None),
            "Cena je povinné pole"
        );
        // Generic message, translated field name.
        assert_eq!(
            full_message(&t, Locale::Cs, "catalog_product", "key", "taken", None),
            "Klíč už je použitý"
        );
        assert_eq!(
            full_message(&t, Locale::Cs, "catalog_variant", "price_czk", "greater_than", Some(0)),
            "Cena musí být větší než 0"
        );
        // No field name: made readable like Rails.
        assert_eq!(attribute_name(&t, Locale::Cs, "x", "price_czk"), "Price czk");
    }

    #[test]
    fn sentences_use_czech_connectors() {
        let t = translations();
        let items = |list: &[&str]| list.iter().map(|item| item.to_string()).collect::<Vec<_>>();
        assert_eq!(to_sentence(&t, Locale::Cs, &items(&["A", "B"])), "A a B");
        assert_eq!(to_sentence(&t, Locale::Cs, &items(&["A", "B", "C"])), "A, B a C");
    }

    #[test]
    fn counts_errors_in_czech() {
        assert_eq!(error_count(1), "1 chyba");
        assert_eq!(error_count(4), "4 chyby");
        assert_eq!(error_count(5), "5 chyb");
    }
}
