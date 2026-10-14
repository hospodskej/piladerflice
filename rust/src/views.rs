//! HTML pages. Templates live in `templates/` and use Jinja syntax
//! (MiniJinja), close to the ERB views of the Rails app:
//!
//! - `{{ t("nav.domu") }}`: a translation (`t("key", km=50)` fills `%{km}`)
//! - `{{ path("/kontakt#cenik") }}`: a site link in the current language
//! - `{{ asset("logo-wm.webp") }}`: the fingerprinted URL of a static file
//!
//! In development templates are re-read when they change, so a template edit
//! needs no recompile; in production they are loaded once at startup.

use std::path::PathBuf;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use minijinja::value::{Kwargs, Value};
use minijinja::{AutoEscape, Environment, Error, ErrorKind, State, context, path_loader};
use minijinja_autoreload::AutoReloader;
use serde::Serialize;
use serde_yaml::Value as Yaml;

use crate::assets::Assets;
use crate::i18n::{self, Translations};
use crate::locale::{self, Locale};
use crate::request::Req;

pub const GA_MEASUREMENT_ID: &str = "G-CMJ46E7F3G";

pub enum Templates {
    Static(Environment<'static>),
    Reloading(AutoReloader),
}

impl Templates {
    pub fn new(dir: PathBuf, translations: Arc<Translations>, assets: Arc<Assets>, reload: bool) -> Self {
        if reload {
            Templates::Reloading(AutoReloader::new(move |notifier| {
                notifier.watch_path(&dir, true);
                Ok(environment(dir.clone(), translations.clone(), assets.clone()))
            }))
        } else {
            Templates::Static(environment(dir, translations, assets))
        }
    }

    pub fn render(&self, name: &str, ctx: Value) -> Result<String, Error> {
        match self {
            Templates::Static(env) => env.get_template(name)?.render(ctx),
            Templates::Reloading(reloader) => reloader.acquire_env()?.get_template(name)?.render(ctx),
        }
    }
}

fn environment(dir: PathBuf, translations: Arc<Translations>, assets: Arc<Assets>) -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(path_loader(dir));
    env.set_trim_blocks(true);
    // Escape like Rails (& < > " '), not "/" too as MiniJinja does by default.
    env.set_formatter(|out, state, value| {
        if matches!(state.auto_escape(), AutoEscape::Html) && !value.is_safe() && value.as_str().is_some() {
            out.write_str(&escape(value.as_str().unwrap_or_default()))
                .map_err(Error::from)
        } else {
            minijinja::escape_formatter(out, state, value)
        }
    });
    env.set_lstrip_blocks(true);

    env.add_function("t", move |state: &State, key: String, kwargs: Kwargs| {
        translate(&translations, state, &key, kwargs)
    });
    env.add_function("path", |state: &State, path: String| {
        Ok::<_, Error>(locale::localized_path(&path, current_locale(state)))
    });
    env.add_function("asset", move |logical: String| assets.url(&logical));
    env
}

fn current_locale(state: &State) -> Locale {
    state
        .lookup("locale")
        .and_then(|value| value.as_str().and_then(Locale::from_code))
        .unwrap_or(Locale::Cs)
}

/// `t("key", name=value)`. Lists and nested sections come back as lists and
/// maps, so templates can loop over them (e.g. the "how to order" steps).
fn translate(translations: &Translations, state: &State, key: &str, kwargs: Kwargs) -> Result<Value, Error> {
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

pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

#[derive(Clone, Debug, Serialize)]
pub struct Crumb {
    pub name: String,
    pub href: String,
}

/// Per-page settings for the layout: `<title>`, description, search engine
/// links and breadcrumbs. Port of `page_meta` in the Rails ApplicationHelper.
#[derive(Clone, Debug, Default)]
pub struct Page {
    pub title: String,
    pub description: Option<String>,
    /// Pages with a description are public pages worth finding in search
    /// engines; they get canonical + hreflang links.
    pub indexable: bool,
    pub noindex: bool,
    /// `None` = derived from the URL; `Some(vec![])` = no breadcrumbs.
    pub breadcrumbs: Option<Vec<Crumb>>,
    pub status: Option<StatusCode>,
    /// Extra trusted HTML for <head> (the JSON-LD business data).
    pub head_html: Option<String>,
}

impl Page {
    /// Title and description from the `meta.<key>` translations.
    pub fn meta(state: &crate::AppState, req: &Req, key: &str) -> Page {
        let title = state.translations.text(req.locale, &format!("meta.{key}.title"), &[]);
        let description = state
            .translations
            .lookup(req.locale, &format!("meta.{key}.description"))
            .and_then(Yaml::as_str)
            .map(str::to_string);
        Page::titled(state, req, &title, description)
    }

    pub fn titled(state: &crate::AppState, req: &Req, title: &str, description: Option<String>) -> Page {
        let site = state.translations.text(req.locale, "site.name", &[]);
        let description = description
            .map(|text| truncate(&squish(&text), 160))
            .filter(|text| !text.is_empty());
        Page {
            title: format!("{title} | {site}"),
            indexable: description.is_some(),
            description,
            ..Page::default()
        }
    }
}

impl Page {
    /// The home page title is the brand line alone, without " | Pila Derflice".
    pub fn brand_only_title(mut self, state: &crate::AppState, req: &Req, key: &str) -> Page {
        self.title = state.translations.text(req.locale, &format!("meta.{key}.title"), &[]);
        self
    }
}

/// Collapses runs of whitespace, like Rails' `squish`.
pub fn squish(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Rails' `truncate(length, separator: " ")`: at most `length` characters,
/// cut at a space, ending in "...".
pub fn truncate(text: &str, length: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= length {
        return text.to_string();
    }
    let stop = length - 3;
    let cut = chars[..=stop.min(chars.len() - 1)]
        .iter()
        .rposition(|c| *c == ' ')
        .unwrap_or(stop);
    let mut out: String = chars[..cut].iter().collect();
    out.push_str("...");
    out
}

/// Breadcrumbs from the URL segments ("/sortiment/palivove-drevo" ->
/// Sortiment > Palivové dřevo).
fn breadcrumbs_from_path(translations: &Translations, req: &Req) -> Vec<Crumb> {
    const NAMES: [(&str, &str); 17] = [
        ("sortiment", "breadcrumbs.sortiment"),
        ("palivove-drevo", "breadcrumbs.palivove_drevo"),
        ("stavebni-rezivo", "breadcrumbs.stavebni_rezivo"),
        ("truhlarske-rezivo", "breadcrumbs.truhlarske_rezivo"),
        ("okrasne-kamenivo", "breadcrumbs.okrasne_kamenivo"),
        ("vyrobni-zbytky", "breadcrumbs.vyrobni_zbytky"),
        ("sluzby", "breadcrumbs.sluzby"),
        ("kontakt", "breadcrumbs.kontakt"),
        ("eshop", "breadcrumbs.eshop"),
        ("kosik", "cart.page_title"),
        ("obchodni-podminky", "legal.terms_title"),
        ("zasady-cookies", "legal.cookies_title"),
        ("zasady-ochrany-osobnich-udaju", "legal.privacy_title"),
        ("prihlaseni", "auth.login_title"),
        ("registrace", "auth.register_title"),
        ("muj-ucet", "auth.account_title"),
        ("upravit", "auth.edit_profile_title"),
    ];
    let mut href = String::new();
    req.plain_path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            href.push('/');
            href.push_str(segment);
            let name = match NAMES.iter().find(|(known, _)| *known == segment) {
                Some((_, key)) => translations.text(req.locale, key, &[]),
                None => capitalize(&segment.replace('-', " ")),
            };
            Crumb {
                name,
                href: req.path_for(&href),
            }
        })
        .collect()
}

fn capitalize(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut chars = lower.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The current page in each language, absolute. Only the query parameters
/// that change what the page shows are kept.
fn language_urls(req: &Req) -> Vec<(Locale, String)> {
    let kept: Vec<(String, String)> = req
        .query_pairs()
        .into_iter()
        .filter(|(key, _)| key == "category")
        .collect();
    let suffix = if kept.is_empty() {
        String::new()
    } else {
        format!(
            "?{}",
            form_urlencoded::Serializer::new(String::new())
                .extend_pairs(kept)
                .finish()
        )
    };
    Locale::ALL
        .into_iter()
        .map(|locale| {
            (
                locale,
                format!(
                    "{}{}{}",
                    req.base_url,
                    locale::localized_path(&req.plain_path, locale),
                    suffix
                ),
            )
        })
        .collect()
}

/// The language switcher: this page in the other language.
fn switch_path(req: &Req, locale: Locale) -> String {
    let path = locale::localized_path(&req.plain_path, locale);
    let kept: Vec<(String, String)> = req
        .query_pairs()
        .into_iter()
        .filter(|(key, _)| key != "locale")
        .collect();
    if kept.is_empty() {
        path
    } else {
        format!(
            "{path}?{}",
            form_urlencoded::Serializer::new(String::new())
                .extend_pairs(kept)
                .finish()
        )
    }
}

/// Renders a page inside the site layout. `data` holds the page's own
/// variables.
pub fn render(state: &crate::AppState, req: &Req, template: &str, page: Page, data: Value) -> Response {
    state.assets.refresh();
    let translations = &state.translations;
    let site_name = translations.text(req.locale, "site.name", &[]);

    let urls = language_urls(req);
    let own_url = urls
        .iter()
        .find(|(locale, _)| *locale == req.locale)
        .map(|(_, url)| url.clone())
        .unwrap_or_default();
    let alternates: Vec<Value> = urls
        .iter()
        .map(|(locale, url)| context! { hreflang => locale.hreflang(), href => url })
        .collect();
    let x_default = urls.first().map(|(_, url)| url.clone()).unwrap_or_default();

    let share_image = ["webp", "png", "jpg", "jpeg"].into_iter().find_map(|ext| {
        let logical = format!("share/share-{}.{ext}", req.locale.code());
        state.assets.exists(&logical).then(|| {
            let mime = if ext == "webp" {
                "image/webp"
            } else if ext == "png" {
                "image/png"
            } else {
                "image/jpeg"
            };
            context! { url => format!("{}{}", req.base_url, state.assets.url(&logical)), mime => mime }
        })
    });

    let breadcrumbs = page
        .breadcrumbs
        .clone()
        .unwrap_or_else(|| breadcrumbs_from_path(translations, req));
    let preloads: Vec<String> = state.assets.importmap().into_iter().map(|(_, url)| url).collect();

    let ctx = context! {
        locale => req.locale.code(),
        current_path => &req.plain_path,
        title => &page.title,
        description => &page.description,
        noindex => page.noindex,
        head_html => page.head_html.clone().map(Value::from_safe_string),
        indexable => page.indexable,
        canonical => own_url,
        alternates => alternates,
        x_default => x_default,
        site_name => &site_name,
        og => context! {
            url => if page.indexable { language_urls(req).into_iter().find(|(l, _)| *l == req.locale).map(|(_, u)| u).unwrap_or_default() } else { format!("{}{}", req.base_url, req.path) },
            locale => req.locale.og_locale(),
            alternate_locales => Locale::ALL.into_iter().filter(|l| *l != req.locale).map(Locale::og_locale).collect::<Vec<_>>(),
            image => share_image,
        },
        breadcrumbs => breadcrumbs,
        switch_cs => switch_path(req, Locale::Cs),
        switch_de => switch_path(req, Locale::De),
        stylesheets => state.assets.stylesheet_urls(),
        importmap_json => state.assets.importmap_json(),
        preloads => preloads,
        ga_measurement_id => GA_MEASUREMENT_ID,
        ..data
    };

    match state.templates.render(template, ctx) {
        Ok(html) => (page.status.unwrap_or(StatusCode::OK), Html(html)).into_response(),
        Err(error) => {
            tracing::error!("rendering {template} failed: {error:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_like_rails() {
        let text = "a ".repeat(100);
        let cut = truncate(text.trim(), 160);
        assert!(cut.chars().count() <= 160);
        assert!(cut.ends_with("a..."));
        assert_eq!(truncate("short", 160), "short");
    }

    #[test]
    fn capitalizes_unknown_segments() {
        assert_eq!(capitalize("neco jineho"), "Neco jineho");
    }
}
