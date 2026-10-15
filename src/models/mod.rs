//! The data the site shows, read from the database. Each struct is one
//! table row; Czech/German column pairs (`title`, `title_de`) are resolved
//! with [`tr`].

pub mod catalog;
pub mod content;
pub mod inquiry;
pub mod inquiry_options;
pub mod login_session;
pub mod order;
pub mod pricelist;
pub mod upload;
pub mod user;

use crate::locale::Locale;

/// The German text when there is one, otherwise the Czech one (the
/// `*_de` columns).
pub fn tr(locale: Locale, cs: &Option<String>, de: &Option<String>) -> String {
    match (locale, de) {
        (Locale::De, Some(de)) if !de.trim().is_empty() => de.clone(),
        _ => cs.clone().unwrap_or_default(),
    }
}

/// "Truhlářské řezivo AB" -> "truhlarske-rezivo-ab": a slug for URLs and ids.
pub fn parameterize(text: &str) -> String {
    let ascii = deunicode::deunicode(text).to_lowercase();
    let mut out = String::with_capacity(ascii.len());
    for c in ascii.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let mut squeezed = String::with_capacity(out.len());
    for c in out.chars() {
        if !(c == '-' && squeezed.ends_with('-')) {
            squeezed.push(c);
        }
    }
    squeezed.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makes_slugs() {
        assert_eq!(parameterize("Truhlářské řezivo AB"), "truhlarske-rezivo-ab");
        assert_eq!(parameterize("  Pořez / prodej kulatiny "), "porez-prodej-kulatiny");
        assert_eq!(parameterize("I. třída"), "i-trida");
    }

    #[test]
    fn german_falls_back_to_czech() {
        let cs = Some("Smrk".to_string());
        assert_eq!(tr(Locale::De, &cs, &Some("Fichte".into())), "Fichte");
        assert_eq!(tr(Locale::De, &cs, &Some(" ".into())), "Smrk");
        assert_eq!(tr(Locale::Cs, &cs, &Some("Fichte".into())), "Smrk");
        assert_eq!(tr(Locale::De, &None, &None), "");
    }
}
