//! The functions templates can call: `t(...)` and `path(...)` (`asset(...)`
//! is a method of the asset index, see `templates.rs`).

use minijinja::value::{Kwargs, Value};
use minijinja::{Error, ErrorKind, State};
use serde_yaml::Value as Yaml;

use super::text::escape;
use crate::i18n::{self, Translations};
use crate::locale::{self, Locale};

/// The language of the page being rendered (the `locale` variable).
pub fn current_locale(state: &State) -> Locale {
    state
        .lookup("locale")
        .and_then(|value| value.as_str().and_then(Locale::from_code))
        .unwrap_or(Locale::Cs)
}

/// `path("/kontakt#cenik")`: a site link in the page's language.
pub fn path(state: &State, path: String) -> Result<String, Error> {
    Ok(locale::localized_path(&path, current_locale(state)))
}

/// `t("key", name=value)`. Lists and nested sections come back as lists and
/// maps, so templates can loop over them (e.g. the "how to order" steps).
pub fn translate(translations: &Translations, state: &State, key: &str, kwargs: Kwargs) -> Result<Value, Error> {
    let html = key.ends_with("_html");
    let mut args = Vec::new();
    for name in kwargs.args() {
        let value: Value = kwargs.get(name)?;
        let text = value.to_string();
        // Like Rails: arguments of an HTML text are escaped unless they are
        // HTML already (a link built in the template).
        args.push((
            name.to_string(),
            if html && !value.is_safe() { escape(&text) } else { text },
        ));
    }
    let args: Vec<(&str, &str)> = args
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();

    let Some(found) = translations.lookup(current_locale(state), key) else {
        tracing::warn!("missing translation {key}");
        return Ok(Value::from(i18n::missing(key)));
    };
    Ok(match found {
        Yaml::String(text) if html => Value::from_safe_string(i18n::interpolate(text, &args)),
        Yaml::String(text) => Value::from(i18n::interpolate(text, &args)),
        other => Value::from_serialize(other),
    })
    .map_err(|error: Error| Error::new(ErrorKind::InvalidOperation, error.to_string()))
}
