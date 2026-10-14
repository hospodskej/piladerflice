//! The breadcrumb trail under the header.

use serde::Serialize;

use super::text::capitalize;
use crate::i18n::Translations;
use crate::request::Req;

#[derive(Clone, Debug, Serialize)]
pub struct Crumb {
    pub name: String,
    pub href: String,
}

/// Breadcrumbs from the URL segments ("/sortiment/palivove-drevo" ->
/// Sortiment > Palivové dřevo).
pub fn from_path(translations: &Translations, req: &Req) -> Vec<Crumb> {
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
