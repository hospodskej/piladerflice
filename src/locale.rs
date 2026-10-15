//! The two languages of the site and their URLs. Czech is the default and has
//! no prefix; the (Austrian) German site lives under `/at`. The translations
//! are German ("de"), the URL prefix names the market.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    Cs,
    De,
}

impl AsRef<str> for Locale {
    fn as_ref(&self) -> &str {
        self.code()
    }
}

impl Locale {
    pub const ALL: [Locale; 2] = [Locale::Cs, Locale::De];

    pub fn code(self) -> &'static str {
        match self {
            Locale::Cs => "cs",
            Locale::De => "de",
        }
    }

    pub fn from_code(code: &str) -> Option<Locale> {
        match code {
            "cs" => Some(Locale::Cs),
            "de" => Some(Locale::De),
            _ => None,
        }
    }

    /// The URL prefix ("at"), or `None` for the unprefixed default language.
    pub fn segment(self) -> Option<&'static str> {
        match self {
            Locale::Cs => None,
            Locale::De => Some("at"),
        }
    }

    pub fn from_segment(segment: &str) -> Option<Locale> {
        Locale::ALL.into_iter().find(|locale| locale.segment() == Some(segment))
    }

    /// The hreflang of the version for search engines: the German site is for Austria.
    pub fn hreflang(self) -> &'static str {
        match self {
            Locale::Cs => "cs",
            Locale::De => "de-AT",
        }
    }

    /// Open Graph locale of the share card.
    pub fn og_locale(self) -> &'static str {
        match self {
            Locale::Cs => "cs_CZ",
            Locale::De => "de_AT",
        }
    }
}

/// The language prefix the path starts with, if any ("/at", "/at/kontakt",
/// "/at?x" and "/at#y" do; "/atlas" doesn't).
pub fn prefix_locale(path: &str) -> Option<Locale> {
    let rest = path.strip_prefix('/')?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    Locale::from_segment(&rest[..end])
}

/// The same path without any language prefix.
pub fn strip(path: &str) -> String {
    let stripped = match prefix_locale(path) {
        Some(locale) => &path[1 + locale.segment().unwrap_or_default().len()..],
        None => path,
    };
    if stripped.starts_with('/') {
        stripped.to_string()
    } else {
        format!("/{stripped}")
    }
}

/// "/kontakt#cenik" -> "/at/kontakt#cenik" in German. The path may carry a
/// query string and an #anchor. Anything that isn't a site path (an
/// absolute URL, "#top", ...) is returned as it is.
pub fn localized_path(path: &str, locale: Locale) -> String {
    if !path.starts_with('/') {
        return path.to_string();
    }
    let base = strip(path);
    match locale.segment() {
        None => base,
        Some(segment) => {
            // "/" and "/?x" become "/at" and "/at?x", not "/at/".
            let rest = match base.strip_prefix('/') {
                Some(after) if after.is_empty() || after.starts_with(['?', '#']) => after,
                _ => base.as_str(),
            };
            format!("/{segment}{rest}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localizes_paths() {
        assert_eq!(localized_path("/kontakt#cenik", Locale::De), "/at/kontakt#cenik");
        assert_eq!(localized_path("/", Locale::De), "/at");
        assert_eq!(
            localized_path("/?category=palivove", Locale::De),
            "/at?category=palivove"
        );
        assert_eq!(localized_path("/at/eshop", Locale::De), "/at/eshop");
        assert_eq!(localized_path("/at/eshop", Locale::Cs), "/eshop");
        assert_eq!(localized_path("/at", Locale::Cs), "/");
        assert_eq!(localized_path("https://example.com", Locale::De), "https://example.com");
    }

    #[test]
    fn strips_only_a_real_prefix() {
        assert_eq!(strip("/at/kontakt"), "/kontakt");
        assert_eq!(strip("/at"), "/");
        assert_eq!(strip("/atlas"), "/atlas");
        assert_eq!(prefix_locale("/at?x=1"), Some(Locale::De));
        assert_eq!(prefix_locale("/kontakt"), None);
    }
}
