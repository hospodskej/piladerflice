//! What every component may need while a page renders: the language, the
//! translations, the asset URLs and the form token. A page provides it once
//! ([`Ui`], see `render.rs`); components call the functions below.
//!
//! ```ignore
//! view! { <a href=path("/kontakt")>{t("nav.kontakt")}</a> }
//! ```

use std::sync::Arc;

use leptos::prelude::*;

use crate::AppState;
use crate::assets::Assets;
use crate::i18n::Translations;
use crate::locale::{self, Locale};
use crate::request::Req;

/// The context of one rendering. Cheap to clone.
#[derive(Clone)]
pub struct Ui(Arc<Inner>);

struct Inner {
    locale: Locale,
    translations: Arc<Translations>,
    assets: Arc<Assets>,
    csrf_token: String,
    /// The URL path without the language prefix ("/kontakt").
    current_path: String,
}

impl Ui {
    pub fn new(state: &AppState, req: &Req) -> Self {
        Self::with(state, req.locale, req.session.csrf_token(), req.plain_path.clone())
    }

    /// Without a request (e-mails): no form token, no current page.
    pub fn with(state: &AppState, locale: Locale, csrf_token: String, current_path: String) -> Self {
        Self(Arc::new(Inner {
            locale,
            translations: state.translations.clone(),
            assets: state.assets.clone(),
            csrf_token,
            current_path,
        }))
    }
}

fn ui() -> Ui {
    expect_context::<Ui>()
}

/// The page's language.
pub fn locale() -> Locale {
    ui().0.locale
}

/// A text in the page's language: `t("nav.domu")`.
pub fn t(key: &str) -> String {
    let ui = ui();
    ui.0.translations.text(ui.0.locale.code(), key, &[])
}

/// A text with values filled in: `t_with("checkout.delivery_line", &[("km", "50")])`.
pub fn t_with(key: &str, args: &[(&str, &str)]) -> String {
    let ui = ui();
    ui.0.translations.text(ui.0.locale.code(), key, args)
}

/// A text that holds trusted HTML (keys ending in `_html`), for `inner_html`.
pub fn t_html(key: &str) -> String {
    t(key)
}

/// An HTML text with values filled in; the values are escaped.
pub fn t_html_with(key: &str, args: &[(&str, &str)]) -> String {
    let escaped: Vec<(&str, String)> = args
        .iter()
        .map(|(name, value)| (*name, sapwood_i18n::escape_html(value)))
        .collect();
    let args: Vec<(&str, &str)> = escaped.iter().map(|(name, value)| (*name, value.as_str())).collect();
    t_with(key, &args)
}

/// A list of texts (a YAML sequence, e.g. the steps of ordering).
pub fn t_list(key: &str) -> Vec<String> {
    let ui = ui();
    ui.0.translations
        .lookup(ui.0.locale.code(), key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// A link inside the site, in the page's language: `path("/kontakt#cenik")`
/// is "/at/kontakt#cenik" on the German site.
pub fn path(path: &str) -> String {
    locale::localized_path(path, locale())
}

/// The fingerprinted URL of a static file: `asset("logo-wm.webp")`.
pub fn asset(logical: &str) -> String {
    ui().0.assets.url(logical)
}

/// The session's form token, for `<input type="hidden" name="csrf_token">`.
pub fn csrf_token() -> String {
    ui().0.csrf_token.clone()
}

/// The page's path without the language prefix.
pub fn current_path() -> String {
    ui().0.current_path.clone()
}

/// The hidden form-protection field every POST form carries.
#[component]
pub fn CsrfField() -> impl IntoView {
    view! { <input type="hidden" name="csrf_token" value=csrf_token()/> }
}
