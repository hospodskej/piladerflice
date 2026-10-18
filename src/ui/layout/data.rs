//! What the layout shows around a page, gathered before rendering: the
//! `<head>` (title, search engine and share links), the signed-in user, the
//! cart, the message of the previous form.

use crate::AppState;
use crate::islands::cart::CartView;
use crate::islands::page::{CONSENT_COOKIE, THEME_COOKIE};
use crate::locale::Locale;
use crate::request::Req;
use crate::session::Flash;
use crate::views::cart_view;
use crate::views::{Crumb, Page, breadcrumbs, seo};

pub const GA_MEASUREMENT_ID: &str = "G-CMJ46E7F3G";

pub struct LayoutData {
    pub locale: Locale,
    pub head: Head,
    pub breadcrumbs: Vec<Crumb>,
    /// This page in Czech and in German (the language switcher).
    pub switch_cs: String,
    pub switch_de: String,
    pub flash: Option<Flash>,
    pub cart: CartView,
    pub user: Option<UserBadge>,
    pub stylesheets: Vec<String>,
    /// Where the islands' WebAssembly is served from; None without a build.
    pub pkg_path: Option<String>,
    /// The visitor chose the dark theme (the `theme` cookie).
    pub dark_mode: bool,
    /// The visitor's cookie choice ("all", "necessary", "declined"); None
    /// until they make one, and the banner asks.
    pub consent: Option<String>,
}

pub struct Head {
    pub title: String,
    pub site_name: String,
    pub description: Option<String>,
    /// Canonical and language links, for pages search engines should find.
    pub links: Option<SearchLinks>,
    pub noindex: bool,
    pub share: ShareCard,
    /// The JSON-LD business data of the home page.
    pub json_ld: Option<String>,
}

pub struct SearchLinks {
    pub canonical: String,
    /// (hreflang, href)
    pub alternates: Vec<(&'static str, String)>,
    pub x_default: String,
}

/// The preview card when a link is shared (Open Graph).
pub struct ShareCard {
    pub url: String,
    pub locale: &'static str,
    pub alternate_locales: Vec<&'static str>,
    pub image: Option<seo::ShareImage>,
}

/// The signed-in user, as the header's account button shows them.
pub struct UserBadge {
    pub avatar_path: Option<String>,
}

impl LayoutData {
    pub async fn load(state: &AppState, req: &Req, page: Page) -> anyhow::Result<Self> {
        state.assets.refresh();
        let urls = seo::language_urls(req);
        let own_url = urls
            .iter()
            .find(|(locale, _)| *locale == req.locale)
            .map(|(_, url)| url.clone())
            .unwrap_or_default();
        let links = page.indexable.then(|| SearchLinks {
            canonical: own_url.clone(),
            alternates: urls
                .iter()
                .map(|(locale, url)| (locale.hreflang(), url.clone()))
                .collect(),
            x_default: urls.first().map(|(_, url)| url.clone()).unwrap_or_default(),
        });
        let share = ShareCard {
            url: if page.indexable {
                own_url
            } else {
                format!("{}{}", req.base_url, req.path)
            },
            locale: req.locale.og_locale(),
            alternate_locales: Locale::ALL
                .into_iter()
                .filter(|locale| *locale != req.locale)
                .map(Locale::og_locale)
                .collect(),
            image: seo::share_image(state, req),
        };
        let site_name = state.translations.text(req.locale, "site.name", &[]);
        let breadcrumbs = page
            .breadcrumbs
            .clone()
            .unwrap_or_else(|| breadcrumbs::from_path(&state.translations, req));
        Ok(LayoutData {
            locale: req.locale,
            head: Head {
                title: if page.title.is_empty() {
                    site_name.clone()
                } else {
                    page.title
                },
                site_name,
                description: page.description,
                links,
                noindex: page.noindex,
                share,
                json_ld: page.json_ld,
            },
            breadcrumbs,
            switch_cs: seo::switch_path(req, Locale::Cs),
            switch_de: seo::switch_path(req, Locale::De),
            flash: page.flash_now.or_else(|| req.session.take_flash()),
            cart: cart_view::load(state, req).await?,
            user: req.user.as_ref().map(|current| UserBadge {
                avatar_path: current.avatar_path(|path| req.path_for(path)),
            }),
            stylesheets: state.assets.urls_with_extension("css"),
            pkg_path: state.wasm.pkg_path(),
            dark_mode: req.cookie(THEME_COOKIE).as_deref() == Some("dark"),
            consent: req.cookie(CONSENT_COOKIE),
        })
    }
}
