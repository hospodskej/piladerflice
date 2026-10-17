//! Links for search engines and the language switcher.

use crate::AppState;
use crate::locale::{self, Locale};
use crate::request::Req;

/// The current page in each language, absolute. Only the query parameters
/// that change what the page shows are kept.
pub fn language_urls(req: &Req) -> Vec<(Locale, String)> {
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
pub fn switch_path(req: &Req, locale: Locale) -> String {
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

/// A share card picture: its absolute URL and type.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ShareImage {
    pub url: String,
    pub mime: &'static str,
}

/// The share card image of the page's language (share/share-cs.webp...),
/// absolute, with its type; None when there is no card for the language.
pub fn share_image(state: &AppState, req: &Req) -> Option<ShareImage> {
    ["webp", "png", "jpg", "jpeg"].into_iter().find_map(|ext| {
        let logical = format!("share/share-{}.{ext}", req.locale.code());
        state.assets.exists(&logical).then(|| {
            let mime = match ext {
                "webp" => "image/webp",
                "png" => "image/png",
                _ => "image/jpeg",
            };
            ShareImage {
                url: format!("{}{}", req.base_url, state.assets.url(&logical)),
                mime,
            }
        })
    })
}
