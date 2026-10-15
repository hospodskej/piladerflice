//! The `t()` function for MiniJinja templates.
//!
//! ```
//! use std::sync::Arc;
//! use sapwood_i18n::{Translations, jinja};
//!
//! let translations = Arc::new(Translations::from_sources([("en", "greeting: Hello {name}\n")]).unwrap());
//! let mut env = minijinja::Environment::new();
//! env.add_function("t", jinja::function(translations, "locale"));
//! env.add_template("page.html", r#"{{ t("greeting", name=user) }}"#).unwrap();
//!
//! let html = env.get_template("page.html").unwrap().render(minijinja::context! { locale => "en", user => "<Pavel>" }).unwrap();
//! assert_eq!(html, "Hello &lt;Pavel&gt;");
//! ```
//!
//! The language is the template variable you name (`locale` above). Lists and
//! sections come back as lists and maps, so a template can loop over them.
//!
//! Keys ending in `_html` hold HTML: the text is not escaped, but its
//! arguments are, unless they are HTML themselves (marked safe, like a link
//! built in the template).

use std::sync::Arc;

use minijinja::value::{Kwargs, Value};
use minijinja::{Error, ErrorKind, State};

use crate::{Translations, escape_html, interpolate, missing};

/// The `t(key, name=value...)` function, reading the language from the
/// template variable `locale_variable`.
pub fn function(
    translations: Arc<Translations>,
    locale_variable: &'static str,
) -> impl Fn(&State, String, Kwargs) -> Result<Value, Error> + Send + Sync + 'static {
    move |state: &State, key: String, kwargs: Kwargs| {
        let locale = state
            .lookup(locale_variable)
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_default();
        translate(&translations, &locale, &key, &kwargs)
    }
}

fn translate(translations: &Translations, locale: &str, key: &str, kwargs: &Kwargs) -> Result<Value, Error> {
    let html = key.ends_with("_html");
    let mut args = Vec::new();
    for name in kwargs.args() {
        let value: Value = kwargs.get(name)?;
        let text = value.to_string();
        args.push((
            name.to_string(),
            if html && !value.is_safe() {
                escape_html(&text)
            } else {
                text
            },
        ));
    }
    let args: Vec<(&str, &str)> = args
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();

    Ok(match translations.lookup(locale, key) {
        None => Value::from(missing(key)),
        Some(serde_json::Value::String(text)) if html => Value::from_safe_string(interpolate(text, &args)),
        Some(serde_json::Value::String(text)) => Value::from(interpolate(text, &args)),
        Some(other) => Value::from_serialize(other),
    })
    .map_err(|error: Error| Error::new(ErrorKind::InvalidOperation, error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(template: &str) -> String {
        let translations = Translations::from_sources([(
            "cs",
            "plain: Ahoj {name}\nlink_html: \"Viz {link}\"\nsteps:\n  - Jedna\n  - Dvě\n",
        )])
        .unwrap();
        let mut env = minijinja::Environment::new();
        env.add_function("t", function(Arc::new(translations), "locale"));
        env.render_str(template, minijinja::context! { locale => "cs" })
            .unwrap()
    }

    #[test]
    fn plain_texts_are_escaped_where_used() {
        assert_eq!(
            render(r#"{% autoescape true %}{{ t("plain", name="<b>") }}{% endautoescape %}"#),
            "Ahoj &lt;b&gt;"
        );
    }

    #[test]
    fn html_texts_escape_only_plain_arguments() {
        assert_eq!(
            render(r#"{% autoescape true %}{{ t("link_html", link="<i>") }}{% endautoescape %}"#),
            "Viz &lt;i&gt;"
        );
        assert_eq!(
            render(r#"{% autoescape true %}{{ t("link_html", link=("<a>x</a>"|safe)) }}{% endautoescape %}"#),
            "Viz <a>x</a>"
        );
    }

    #[test]
    fn lists_can_be_looped_over() {
        assert_eq!(render(r#"{% for s in t("steps") %}{{ s }};{% endfor %}"#), "Jedna;Dvě;");
        assert_eq!(render(r#"{{ t("nope") }}"#), "[missing: nope]");
    }
}
