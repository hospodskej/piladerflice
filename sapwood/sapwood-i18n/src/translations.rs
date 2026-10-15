//! The translations of every language, and looking keys up.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;

use crate::{Error, interpolate};

/// Every language's translations: a tree of keys per language code.
#[derive(Debug, Default, Clone)]
pub struct Translations {
    locales: BTreeMap<String, Value>,
    fallback: Option<String>,
}

impl Translations {
    /// Reads every `<locale>.yml` (or `.yaml`) file in `dir`: `cs.yml` holds
    /// the `cs` translations. Other files are left alone.
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Self, Error> {
        let dir = dir.as_ref();
        let entries = std::fs::read_dir(dir).map_err(|source| Error::Read {
            file: dir.to_path_buf(),
            source,
        })?;
        let mut sources = Vec::new();
        for entry in entries {
            let file = entry
                .map_err(|source| Error::Read {
                    file: dir.to_path_buf(),
                    source,
                })?
                .path();
            let is_yaml = matches!(file.extension().and_then(|ext| ext.to_str()), Some("yml" | "yaml"));
            let Some(locale) = file.file_stem().and_then(|stem| stem.to_str()).filter(|_| is_yaml) else {
                continue;
            };
            let text = std::fs::read_to_string(&file).map_err(|source| Error::Read {
                file: file.clone(),
                source,
            })?;
            sources.push((locale.to_string(), text));
        }
        Self::from_sources(sources)
    }

    /// Translations from YAML texts: `(locale, yaml)` pairs.
    pub fn from_sources<L, S>(sources: impl IntoIterator<Item = (L, S)>) -> Result<Self, Error>
    where
        L: Into<String>,
        S: AsRef<str>,
    {
        let mut locales = BTreeMap::new();
        for (locale, yaml) in sources {
            let locale = locale.into();
            let tree: Value = serde_saphyr::from_str(yaml.as_ref()).map_err(|error| Error::Parse {
                locale: locale.clone(),
                message: error.to_string(),
            })?;
            let tree = match tree {
                Value::Object(_) => tree,
                Value::Null => Value::Object(Default::default()),
                _ => return Err(Error::NotAMap { locale }),
            };
            locales.insert(locale, tree);
        }
        Ok(Self {
            locales,
            fallback: None,
        })
    }

    /// Keys missing in a language are looked up in `locale` instead.
    pub fn with_fallback(mut self, locale: impl Into<String>) -> Self {
        self.fallback = Some(locale.into());
        self
    }

    /// The language codes there are translations for.
    pub fn locales(&self) -> impl Iterator<Item = &str> {
        self.locales.keys().map(String::as_str)
    }

    /// The text, list or section at `key` ("nav.home"), from the fallback
    /// language when `locale` doesn't have it. `locale` is a language code,
    /// or your own language type if it implements `AsRef<str>`.
    pub fn lookup(&self, locale: impl AsRef<str>, key: &str) -> Option<&Value> {
        let locale = locale.as_ref();
        self.lookup_in(locale, key).or_else(|| {
            self.fallback
                .as_deref()
                .filter(|fallback| *fallback != locale)
                .and_then(|fallback| self.lookup_in(fallback, key))
        })
    }

    /// Whether `locale` itself has `key` (the fallback doesn't count): for
    /// tests that check every key is translated.
    pub fn has(&self, locale: impl AsRef<str>, key: &str) -> bool {
        self.lookup_in(locale.as_ref(), key).is_some()
    }

    /// A plain text with its `{name}` placeholders filled in. A missing key,
    /// or one that isn't a text, comes back as `[missing: key]`.
    pub fn text(&self, locale: impl AsRef<str>, key: &str, args: &[(&str, &str)]) -> String {
        match self.lookup(locale, key).and_then(Value::as_str) {
            Some(text) => interpolate(text, args),
            None => missing(key),
        }
    }

    fn lookup_in(&self, locale: &str, key: &str) -> Option<&Value> {
        let mut node = self.locales.get(locale)?;
        for part in key.split('.') {
            node = node.get(part)?;
        }
        (!node.is_null()).then_some(node)
    }
}

/// What a missing translation shows: `[missing: key]`, easy to spot on a page.
pub fn missing(key: &str) -> String {
    format!("[missing: {key}]")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn translations() -> Translations {
        Translations::from_sources([
            (
                "cs",
                "nav:\n  home: Domů\n  steps:\n    - Vyberte\n    - Objednejte\nempty:\n",
            ),
            ("de", "nav:\n  home: Startseite\n"),
        ])
        .unwrap()
        .with_fallback("cs")
    }

    #[test]
    fn looks_up_dotted_keys() {
        let t = translations();
        assert_eq!(t.text("cs", "nav.home", &[]), "Domů");
        assert_eq!(t.text("de", "nav.home", &[]), "Startseite");
        assert_eq!(t.lookup("cs", "nav.steps").unwrap().as_array().unwrap().len(), 2);
    }

    #[test]
    fn falls_back_but_has_does_not() {
        let t = translations();
        assert_eq!(t.lookup("de", "nav.steps").unwrap().as_array().unwrap().len(), 2);
        assert!(t.has("cs", "nav.steps"));
        assert!(!t.has("de", "nav.steps"));
    }

    #[test]
    fn missing_and_empty_keys_are_marked() {
        let t = translations();
        assert_eq!(t.text("cs", "nav.nowhere", &[]), "[missing: nav.nowhere]");
        assert_eq!(t.text("cs", "empty", &[]), "[missing: empty]");
        assert_eq!(t.text("cs", "nav", &[]), "[missing: nav]");
        assert_eq!(t.text("xx", "nav.home", &[]), "Domů");
    }

    #[test]
    fn reports_bad_files() {
        assert!(matches!(
            Translations::from_sources([("cs", "- a list")]),
            Err(Error::NotAMap { .. })
        ));
        assert!(matches!(
            Translations::from_sources([("cs", "a: [unclosed")]),
            Err(Error::Parse { .. })
        ));
    }

    #[test]
    fn loads_a_directory() {
        let dir = std::env::temp_dir().join(format!("sapwood-i18n-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("en.yml"), "hello: Hello\n").unwrap();
        std::fs::write(dir.join("README.md"), "not translations").unwrap();
        let t = Translations::load_dir(&dir).unwrap();
        assert_eq!(t.locales().collect::<Vec<_>>(), ["en"]);
        assert_eq!(t.text("en", "hello", &[]), "Hello");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
