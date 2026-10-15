//! Translations, read from `locales/{cs,de}.yml` (the format of Rails' I18n
//! files, which they come from).
//!
//! Like Rails I18n: keys are dotted paths ("nav.domu"), `%{name}` is
//! replaced by an argument, a missing German text falls back to Czech, and
//! keys ending in `_html` are trusted HTML (their arguments are escaped).

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde_yaml::Value as Yaml;

use crate::locale::Locale;

pub struct Translations {
    roots: HashMap<&'static str, Yaml>,
}

impl Translations {
    pub fn load(dir: &Path) -> Result<Self> {
        let mut roots = HashMap::new();
        for locale in Locale::ALL {
            let file = dir.join(format!("{}.yml", locale.code()));
            let text = std::fs::read_to_string(&file).with_context(|| format!("reading {}", file.display()))?;
            let mut document: Yaml =
                serde_yaml::from_str(&text).with_context(|| format!("parsing {}", file.display()))?;
            let root = document
                .get_mut(locale.code())
                .map(std::mem::take)
                .with_context(|| format!("{} has no top-level `{}:` key", file.display(), locale.code()))?;
            roots.insert(locale.code(), root);
        }
        Ok(Self { roots })
    }

    /// The raw value at `key` (a string, a list or a nested map), falling back
    /// to Czech when the German text is missing.
    pub fn lookup(&self, locale: Locale, key: &str) -> Option<&Yaml> {
        self.lookup_in(locale, key).or_else(|| match locale {
            Locale::Cs => None,
            _ => self.lookup_in(Locale::Cs, key),
        })
    }

    /// Whether this language itself has the key (no falling back to Czech).
    pub fn has(&self, locale: Locale, key: &str) -> bool {
        self.lookup_in(locale, key).is_some()
    }

    fn lookup_in(&self, locale: Locale, key: &str) -> Option<&Yaml> {
        let mut node = self.roots.get(locale.code())?;
        for part in key.split('.') {
            node = node.get(part)?;
        }
        (!node.is_null()).then_some(node)
    }

    /// A plain text (not HTML), with `%{name}` replaced. Missing keys come back
    /// as "[missing: key]" so they stand out on the page and in tests.
    pub fn text(&self, locale: Locale, key: &str, args: &[(&str, &str)]) -> String {
        match self.lookup(locale, key).and_then(Yaml::as_str) {
            Some(text) => interpolate(text, args),
            None => missing(key),
        }
    }
}

pub fn missing(key: &str) -> String {
    format!("[missing: {key}]")
}

/// Replaces each `%{name}` with its argument; unknown names stay as they are.
pub fn interpolate(text: &str, args: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("%{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find('}') {
            Some(end) => {
                let name = &after[..end];
                match args.iter().find(|(arg, _)| *arg == name) {
                    Some((_, value)) => out.push_str(value),
                    None => out.push_str(&rest[start..start + 3 + end]),
                }
                rest = &after[end + 1..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_named_arguments() {
        assert_eq!(
            interpolate("%{km} km po %{rate}", &[("km", "50"), ("rate", "40 Kč")]),
            "50 km po 40 Kč"
        );
        assert_eq!(interpolate("100 %{unknown}", &[]), "100 %{unknown}");
        assert_eq!(interpolate("open %{", &[]), "open %{");
    }
}
