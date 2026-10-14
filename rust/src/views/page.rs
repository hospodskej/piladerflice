//! Per-page settings for the layout. Port of `page_meta` in the Rails
//! ApplicationHelper.

use axum::http::StatusCode;
use serde_yaml::Value as Yaml;

use super::breadcrumbs::Crumb;
use super::text::{squish, truncate};
use crate::AppState;
use crate::request::Req;

/// `<title>`, description, search engine links and breadcrumbs of a page.
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
    pub fn meta(state: &AppState, req: &Req, key: &str) -> Page {
        let title = state.translations.text(req.locale, &format!("meta.{key}.title"), &[]);
        let description = state
            .translations
            .lookup(req.locale, &format!("meta.{key}.description"))
            .and_then(Yaml::as_str)
            .map(str::to_string);
        Page::titled(state, req, &title, description)
    }

    pub fn titled(state: &AppState, req: &Req, title: &str, description: Option<String>) -> Page {
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
    pub fn brand_only_title(mut self, state: &AppState, req: &Req, key: &str) -> Page {
        self.title = state.translations.text(req.locale, &format!("meta.{key}.title"), &[]);
        self
    }
}
