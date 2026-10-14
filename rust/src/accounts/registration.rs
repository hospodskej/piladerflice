//! Checking a new account, with Rails' messages and in Rails' order:
//! password missing, confirmation not matching, e-mail missing or taken,
//! password too short.

use crate::i18n::Translations;
use crate::locale::Locale;
use crate::models::validation::full_message;

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
    let message = |field: &str, error: &str, count: Option<usize>| {
        full_message(translations, locale, "user", field, error, count.map(|c| c as i64))
    };
    let mut errors = Vec::new();
    if registration.password.is_empty() {
        errors.push(message("password", "blank", None));
    }
    if registration.password.len() > MAX_PASSWORD_BYTES {
        let attribute = translations.text(locale, "activerecord.attributes.user.password", &[]);
        errors.push(format!(
            "{attribute} is too long (maximum is {MAX_PASSWORD_BYTES} bytes)"
        ));
    }
    if let Some(confirmation) = registration.confirmation
        && confirmation != registration.password
    {
        errors.push(message("password_confirmation", "confirmation", None));
    }
    if registration.email.trim().is_empty() {
        errors.push(message("email_address", "blank", None));
    } else if registration.email_taken {
        errors.push(message("email_address", "taken", None));
    }
    if !registration.password.is_empty() && registration.password.chars().count() < MIN_PASSWORD {
        errors.push(message("password", "too_short", Some(MIN_PASSWORD)));
    }
    errors
}
