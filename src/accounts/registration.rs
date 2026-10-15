//! Checking a new account: password missing or too long, confirmation not
//! matching, e-mail missing or taken, password too short.

use crate::i18n::Translations;
use crate::locale::Locale;

pub const MIN_PASSWORD: usize = 8;
/// bcrypt only looks at the first 72 bytes.
pub const MAX_PASSWORD_BYTES: usize = 72;

pub struct Registration<'a> {
    pub email: &'a str,
    pub password: &'a str,
    pub confirmation: Option<&'a str>,
    pub email_taken: bool,
}

/// "Heslo je příliš krátké (nejméně 8 znaků)", ...; empty when all is well.
pub fn errors(registration: &Registration, translations: &Translations, locale: Locale) -> Vec<String> {
    let message = |key: &str, count: Option<usize>| {
        let count = count.map(|count| count.to_string()).unwrap_or_default();
        translations.text(locale, &format!("validation.registration.{key}"), &[("count", &count)])
    };
    let mut errors = Vec::new();
    if registration.password.is_empty() {
        errors.push(message("password_blank", None));
    }
    if registration.password.len() > MAX_PASSWORD_BYTES {
        errors.push(message("password_too_long", Some(MAX_PASSWORD_BYTES)));
    }
    if let Some(confirmation) = registration.confirmation
        && confirmation != registration.password
    {
        errors.push(message("password_confirmation", None));
    }
    if registration.email.trim().is_empty() {
        errors.push(message("email_blank", None));
    } else if registration.email_taken {
        errors.push(message("email_taken", None));
    }
    if !registration.password.is_empty() && registration.password.chars().count() < MIN_PASSWORD {
        errors.push(message("password_too_short", Some(MIN_PASSWORD)));
    }
    errors
}
