//! Languages. The Czech site has no prefix; the German (Austrian) one lives
//! under `/at`. The language comes from the address only, so a shared link
//! opens in the language it was shared in.

use std::borrow::Cow;

// `Locale` (cs, de) and `t::...`, one typed function per translation key,
// generated from locales/*.yml by build.rs.
codon::include!();

impl Locale {
    /// The language a site path is in.
    pub fn of_path(path: &str) -> Locale {
        Locale::PATHS
            .detect(path)
            .map(|(locale, _)| locale)
            .unwrap_or_default()
    }

    /// `path` in this language: "/kontakt#cenik" → "/at/kontakt#cenik".
    pub fn path(self, path: &str) -> String {
        Locale::PATHS.localize(path, self).into_owned()
    }

    /// The other language (the header's language switch).
    pub fn other(self) -> Locale {
        match self {
            Locale::Cs => Locale::De,
            Locale::De => Locale::Cs,
        }
    }

    /// "de-AT" for hreflang: the German site is for Austria.
    pub fn hreflang(self) -> &'static str {
        match self {
            Locale::Cs => "cs",
            Locale::De => "de-AT",
        }
    }

    /// The Open Graph locale.
    pub fn og_locale(self) -> &'static str {
        match self {
            Locale::Cs => "cs_CZ",
            Locale::De => "de_AT",
        }
    }

    /// The currency the header shows for the language.
    pub fn currency(self) -> &'static str {
        match self {
            Locale::Cs => "CZK",
            Locale::De => "EUR",
        }
    }
}

/// The same path without a language prefix: "/at/kontakt" → "/kontakt",
/// "/at" → "/", "/at?x=1" → "/?x=1", "/atlas" stays.
pub fn strip_prefix(path: &str) -> Cow<'_, str> {
    Locale::PATHS.strip(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths() {
        assert_eq!(Locale::De.path("/kontakt#cenik"), "/at/kontakt#cenik");
        assert_eq!(Locale::De.path("/"), "/at");
        assert_eq!(Locale::De.path("/?category=rezivo"), "/at?category=rezivo");
        assert_eq!(Locale::De.path("/at/eshop"), "/at/eshop");
        assert_eq!(Locale::Cs.path("/at/eshop/tramy"), "/eshop/tramy");
        assert_eq!(Locale::Cs.path("/at"), "/");
        assert_eq!(Locale::Cs.path("/atlas"), "/atlas");
        assert_eq!(Locale::of_path("/at/kontakt"), Locale::De);
        assert_eq!(Locale::of_path("/at"), Locale::De);
        assert_eq!(Locale::of_path("/atlas"), Locale::Cs);
        assert_eq!(strip_prefix("/at?x=1"), "/?x=1");
    }
}
