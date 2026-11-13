//! Who is looking at the page: language, signed-in user, cart, delivery
//! estimate and the one-off message from the last action. Loaded once per
//! request on the server (see `session::load_visitor`) and read by the pages.

use crate::i18n::Locale;
use domain::cart::Cart;
use domain::delivery::DeliveryEstimate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedInUser {
    pub id: i64,
    pub email: String,
    pub first_name: Option<String>,
    pub admin: bool,
    pub theme: Option<Theme>,
    /// The profile picture's address, if there is one.
    pub avatar: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    pub fn parse(value: &str) -> Option<Theme> {
        match value {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
        }
    }
}

/// A message shown once at the top of the next page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Flash {
    Notice(String),
    Alert(String),
}

#[derive(Clone, Debug, Default)]
pub struct Visitor {
    pub locale: Locale,
    /// The request path, with its language prefix.
    pub path: String,
    pub query: Option<String>,
    pub user: Option<SignedInUser>,
    pub cart: Cart,
    pub delivery: Option<DeliveryEstimate>,
    pub flash: Option<Flash>,
    pub shop: crate::shop::Shop,
}

impl Visitor {
    /// `path` in the visitor's language.
    pub fn link(&self, path: &str) -> String {
        self.locale.path(path)
    }

    /// The current page in the other language (the header's language switch).
    pub fn switch_language_link(&self, locale: Locale) -> String {
        let path = locale.path(&crate::i18n::strip_prefix(&self.path));
        match &self.query {
            Some(q) if !q.is_empty() => format!("{path}?{q}"),
            _ => path,
        }
    }

    /// Whether the page is `path` (unprefixed), for marking the active link.
    pub fn is_at(&self, path: &str) -> bool {
        crate::i18n::strip_prefix(&self.path) == path
    }
}

/// The current visitor. On the server it's loaded for each request; in the
/// browser (islands) there's none, so islands get what they need as props.
pub fn visitor() -> Visitor {
    #[cfg(feature = "ssr")]
    {
        if let Some(parts) = leptos::prelude::use_context::<axum::http::request::Parts>() {
            if let Some(v) = parts.extensions.get::<Visitor>() {
                return v.clone();
            }
        }
    }
    Visitor::default()
}
