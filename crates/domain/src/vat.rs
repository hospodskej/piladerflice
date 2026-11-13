//! VAT by category and destination country. Czech 21 % on everything;
//! Austria has its own rates per category. Admin prices are either with VAT
//! already in them or without (then the customer pays the Czech rate on top).
//! Discounts and delivery are split across the VAT rates by the goods' share.

use crate::catalog::{text_enum, Category};
use crate::money::{round_div, Czk};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

text_enum! {
    /// The country whose VAT applies (the delivery country; pickup is Czech).
    Country {
        Cz => "cz",
        At => "at",
    }
}

impl Country {
    /// Anything unknown or missing counts as the Czech Republic.
    pub fn or_cz(value: Option<&str>) -> Country {
        value.and_then(|v| v.parse().ok()).unwrap_or(Country::Cz)
    }
}

/// The VAT rule of one category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VatRule {
    pub category: Category,
    pub rate_cz: u8,
    pub rate_at: u8,
    /// The admin enters this category's prices with VAT in them.
    pub prices_include_vat: bool,
}

impl VatRule {
    pub const DEFAULT_RATE_CZ: u8 = 21;
    pub const DEFAULT_RATE_AT: u8 = 20;

    /// The rule when the admin hasn't set one: prices with VAT, standard rates.
    pub fn default_for(category: Category) -> VatRule {
        VatRule { category, rate_cz: Self::DEFAULT_RATE_CZ, rate_at: Self::DEFAULT_RATE_AT, prices_include_vat: true }
    }

    pub fn rate_for(&self, country: Country) -> u8 {
        match country {
            Country::Cz => self.rate_cz,
            Country::At => self.rate_at,
        }
    }

    /// The price the customer sees, from the price the admin entered.
    /// It's the same in both countries; only the VAT inside it differs.
    pub fn customer_price(&self, admin_price: Czk) -> Czk {
        if self.prices_include_vat { admin_price } else { admin_price.plus_vat(self.rate_cz) }
    }
}

/// All the categories' rules, falling back to [`VatRule::default_for`].
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VatRules(BTreeMap<Category, VatRule>);

impl VatRules {
    pub fn new(rules: impl IntoIterator<Item = VatRule>) -> VatRules {
        VatRules(rules.into_iter().map(|r| (r.category, r)).collect())
    }

    pub fn get(&self, category: Category) -> VatRule {
        self.0.get(&category).copied().unwrap_or_else(|| VatRule::default_for(category))
    }

    pub fn customer_price(&self, category: Category, admin_price: Czk) -> Czk {
        self.get(category).customer_price(admin_price)
    }

    /// The amount without VAT, for showing "excl. VAT" next to a price.
    pub fn excl_vat(&self, category: Category, amount: Czk, country: Country) -> Czk {
        amount - amount.vat_in(self.get(category).rate_for(country))
    }
}

/// The VAT of a whole order: per rate, after discount and delivery.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VatBreakdown {
    pub country: Country,
    /// VAT per rate, highest rate first.
    pub by_rate: Vec<(u8, Czk)>,
    pub goods: Czk,
    pub discount: Czk,
    pub delivery: Czk,
}

impl VatBreakdown {
    /// `lines` are (category, VAT-inclusive line total).
    pub fn new(
        rules: &VatRules,
        lines: impl IntoIterator<Item = (Category, Czk)>,
        country: Country,
        discount: Czk,
        delivery: Czk,
    ) -> VatBreakdown {
        // Goods per rate, in the order the rates first appear (the last one
        // takes the rounding remainder, as before).
        let mut goods_by_rate: Vec<(u8, Czk)> = Vec::new();
        for (category, amount) in lines {
            let rate = rules.get(category).rate_for(country);
            match goods_by_rate.iter_mut().find(|(r, _)| *r == rate) {
                Some((_, sum)) => *sum += amount,
                None => goods_by_rate.push((rate, amount)),
            }
        }
        let goods: Czk = goods_by_rate.iter().map(|(_, a)| *a).sum();
        let extra = delivery - discount;

        let mut allocated = Czk::ZERO;
        let last = goods_by_rate.len().saturating_sub(1);
        let mut by_rate: Vec<(u8, Czk)> = goods_by_rate
            .iter()
            .enumerate()
            .map(|(i, &(rate, amount))| {
                let share = if i == last {
                    extra - allocated
                } else if goods == Czk::ZERO {
                    Czk::ZERO
                } else {
                    Czk(round_div(extra.get() * amount.get(), goods.get()))
                };
                allocated += share;
                (rate, amount + share)
            })
            .filter(|(_, amount)| *amount != Czk::ZERO)
            .map(|(rate, amount)| (rate, amount.vat_in(rate)))
            .collect();
        by_rate.sort_by(|a, b| b.0.cmp(&a.0));

        VatBreakdown { country, by_rate, goods, discount, delivery }
    }

    pub fn vat(&self) -> Czk {
        self.by_rate.iter().map(|(_, v)| *v).sum()
    }

    pub fn total(&self) -> Czk {
        self.goods - self.discount + self.delivery
    }

    pub fn total_excl_vat(&self) -> Czk {
        self.total() - self.vat()
    }

    /// {"21": 174, ...}, the form stored on the order.
    pub fn to_json(&self) -> String {
        let map: BTreeMap<String, i64> = self.by_rate.iter().map(|(r, v)| (r.to_string(), v.get())).collect();
        serde_json::to_string(&map).unwrap_or_else(|_| "{}".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> VatRules {
        VatRules::new([
            VatRule { category: Category::Firewood, rate_cz: 21, rate_at: 13, prices_include_vat: true },
            VatRule { category: Category::Lumber, rate_cz: 21, rate_at: 20, prices_include_vat: false },
        ])
    }

    #[test]
    fn customer_price_adds_czech_vat_to_net_prices() {
        let r = rules();
        assert_eq!(r.customer_price(Category::Lumber, Czk(1000)), Czk(1210));
        assert_eq!(r.customer_price(Category::Firewood, Czk(1000)), Czk(1000));
        // no rule set: prices include VAT
        assert_eq!(r.customer_price(Category::Stone, Czk(1000)), Czk(1000));
    }

    #[test]
    fn czech_order_is_one_rate() {
        let b = VatBreakdown::new(
            &rules(),
            [(Category::Firewood, Czk(1210)), (Category::Lumber, Czk(1210))],
            Country::Cz,
            Czk::ZERO,
            Czk::ZERO,
        );
        assert_eq!(b.by_rate, vec![(21, Czk(420))]);
        assert_eq!(b.total(), Czk(2420));
        assert_eq!(b.total_excl_vat(), Czk(2000));
    }

    #[test]
    fn austrian_order_splits_discount_and_delivery_by_share() {
        // goods: 13 % -> 3000, 20 % -> 1000; extra = 400 delivery - 200 discount = 200
        // 13 % share = round(200 * 3000 / 4000) = 150, 20 % takes the remaining 50
        let b = VatBreakdown::new(
            &rules(),
            [(Category::Firewood, Czk(3000)), (Category::Lumber, Czk(1000))],
            Country::At,
            Czk(200),
            Czk(400),
        );
        assert_eq!(b.by_rate, vec![(20, Czk(1050).vat_in(20)), (13, Czk(3150).vat_in(13))]);
        assert_eq!(b.total(), Czk(4200));
        assert_eq!(b.to_json(), format!("{{\"13\":{},\"20\":{}}}", Czk(3150).vat_in(13).get(), Czk(1050).vat_in(20).get()));
    }

    #[test]
    fn empty_cart_has_no_vat() {
        let b = VatBreakdown::new(&rules(), [], Country::Cz, Czk::ZERO, Czk::ZERO);
        assert!(b.by_rate.is_empty());
        assert_eq!(b.vat(), Czk::ZERO);
    }

    #[test]
    fn unknown_country_is_czech() {
        assert_eq!(Country::or_cz(Some("de")), Country::Cz);
        assert_eq!(Country::or_cz(None), Country::Cz);
        assert_eq!(Country::or_cz(Some("at")), Country::At);
    }
}
