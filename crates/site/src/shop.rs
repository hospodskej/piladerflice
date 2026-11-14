//! Shop settings every page needs: VAT rules, the price per km, the euro
//! rate and the product names, loaded once per request.

use crate::i18n::{Locale, t};
use domain::cart::Cart;
use domain::cart::{CartLine, Spec};
use domain::money::{Czk, EurRate};
use domain::vat::{Country, VatBreakdown, VatRules};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Shop {
    pub vat: VatRules,
    pub czk_per_km: i64,
    pub eur: EurRate,
    /// Product key → (Czech title, German title).
    pub titles: HashMap<String, (String, Option<String>)>,
}

impl Default for Shop {
    fn default() -> Shop {
        Shop {
            vat: VatRules::default(),
            czk_per_km: domain::delivery::DEFAULT_CZK_PER_KM,
            eur: EurRate::FALLBACK,
            titles: HashMap::new(),
        }
    }
}

/// Text the admin entered in Czech and maybe German: the German one on the
/// German site when there is one.
pub fn translated<'a>(locale: Locale, cs: &'a str, de: Option<&'a str>) -> &'a str {
    match (locale, de) {
        (Locale::De, Some(de)) if !de.trim().is_empty() => de,
        _ => cs,
    }
}

impl Shop {
    /// "1 740 Kč", or "≈ 69,60 €" on the German site.
    pub fn price(&self, locale: Locale, amount: Czk) -> String {
        match locale {
            Locale::Cs => amount.to_string(),
            Locale::De => self.eur.approx_eur(amount),
        }
    }

    /// An approximate amount: "≈ 640 Kč", or "≈ 25,60 €" (never "≈ ≈").
    pub fn approx_price(&self, locale: Locale, amount: Czk) -> String {
        let price = self.price(locale, amount);
        if price.starts_with('≈') {
            price
        } else {
            format!("≈ {price}")
        }
    }

    /// The amount alone, for texts that already say "≈ %{price}".
    pub fn bare_price(&self, locale: Locale, amount: Czk) -> String {
        self.price(locale, amount)
            .trim_start_matches("≈ ")
            .to_owned()
    }

    /// An admin-written price text ("od 1 740 Kč / m³"), in euros on the German site.
    pub fn price_text(&self, locale: Locale, text: &str) -> String {
        match locale {
            Locale::Cs => text.to_owned(),
            Locale::De => self.eur.convert_price_text(text),
        }
    }

    pub fn product_title(&self, locale: Locale, key: &str) -> String {
        match self.titles.get(key) {
            Some((cs, de)) => translated(locale, cs, de.as_deref()).to_owned(),
            // A product no longer sold (an old order): its key made readable.
            None => {
                let text = key.replace(['_', '-'], " ");
                let mut chars = text.chars();
                chars
                    .next()
                    .map(|c| c.to_uppercase().chain(chars).collect())
                    .unwrap_or_default()
            }
        }
    }

    /// "Buk, Volně ložené, 33 cm, 4 prm".
    pub fn line_description(&self, locale: Locale, line: &CartLine) -> String {
        std::iter::once(self.product_title(locale, &line.product_key))
            .chain(line.specs.iter().map(|spec| spec_text(locale, spec)))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn breakdown(
        &self,
        cart: &Cart,
        country: Country,
        discount: Czk,
        delivery: Czk,
    ) -> VatBreakdown {
        VatBreakdown::new(&self.vat, cart.vat_lines(), country, discount, delivery)
    }
}

pub fn spec_text(locale: Locale, spec: &Spec) -> String {
    match spec {
        Spec::Key { value } => crate::i18n::lookup(locale, value)
            .unwrap_or(value)
            .to_owned(),
        Spec::Raw { value } => value.clone(),
        Spec::Amount { value, unit_key } => {
            format!(
                "{value} {}",
                crate::i18n::lookup(locale, unit_key).unwrap_or(unit_key)
            )
        }
        Spec::Bilingual { value, value_de } => {
            translated(locale, value, value_de.as_deref()).to_owned()
        }
    }
}

/// "Košík" or "Košík, počet kusů: 3".
pub fn cart_label(locale: Locale, count: u32) -> String {
    if count > 0 {
        t::cart::aria_label_count(locale, count)
    } else {
        t::cart::aria_label(locale).to_owned()
    }
}

#[cfg(feature = "ssr")]
pub async fn load(state: &crate::state::AppState) -> Result<Shop, sqlx::Error> {
    use domain::Category;
    use domain::vat::VatRule;

    let vat = sqlx::query!("SELECT category, rate_cz, rate_at, prices_include_vat FROM vat_rules")
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .filter_map(|r| {
            Some(VatRule {
                category: r.category.parse::<Category>().ok()?,
                rate_cz: u8::try_from(r.rate_cz).ok()?,
                rate_at: u8::try_from(r.rate_at).ok()?,
                prices_include_vat: r.prices_include_vat != 0,
            })
        });
    let czk_per_km =
        sqlx::query_scalar!("SELECT value FROM settings WHERE key = 'delivery_czk_per_km'")
            .fetch_optional(&state.db)
            .await?
            .and_then(|v| v.parse().ok())
            .unwrap_or(domain::delivery::DEFAULT_CZK_PER_KM);
    let titles = sqlx::query!("SELECT key, title, title_de FROM catalog_products")
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|r| (r.key, (r.title, r.title_de)))
        .collect();
    Ok(Shop {
        vat: VatRules::new(vat),
        czk_per_km,
        eur: state.eur_rate(),
        titles,
    })
}
