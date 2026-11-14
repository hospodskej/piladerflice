//! The cart as the pages show it: lines and total rows, worked out on the
//! server (prices, VAT, discount, delivery estimate) and handed to the
//! islands as plain data.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RowKind {
    /// A grey row (without VAT, VAT, goods before discount).
    Muted,
    Discount,
    Delivery,
    /// "Celkem k úhradě", in bold.
    Total,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TotalRow {
    pub label: String,
    pub value: String,
    pub kind: RowKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineView {
    pub variant_id: i64,
    pub image: Option<String>,
    pub description: String,
    pub quantity: u32,
    pub unit_price: String,
    pub total: String,
    pub total_excl_vat: String,
}

/// What the header's cart dropdown shows.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CartSummary {
    pub count: u32,
    /// The button's accessible name ("Košík, počet kusů: 3").
    pub label: String,
    pub lines: Vec<LineView>,
    pub totals: Vec<TotalRow>,
    pub empty_text: String,
    pub view_cart_text: String,
    pub view_cart_href: String,
}

#[cfg(feature = "ssr")]
mod build {
    use super::*;
    use crate::i18n::t;
    use crate::visitor::Visitor;
    use domain::money::Czk;
    use domain::vat::Country;

    /// Whose VAT applies, as far as we know yet: checkout's (pickup is
    /// Czech; else where the goods go), then "Kam vozíme?", then the site's
    /// language (the German site delivers to Austria by default).
    pub fn vat_country(v: &Visitor) -> Country {
        v.checkout
            .vat_country()
            .or_else(|| v.delivery.as_ref().map(|d| d.country))
            .unwrap_or(match v.locale {
                crate::i18n::Locale::De => Country::At,
                crate::i18n::Locale::Cs => Country::Cz,
            })
    }

    /// The cart's code and what it takes off now, if it can be used.
    pub fn applied_discount(v: &Visitor) -> Option<(String, Czk)> {
        let code = v.discount.as_ref()?;
        let subtotal = v.cart.subtotal();
        code.unusable_reason(subtotal, time::OffsetDateTime::now_utc().date())
            .is_none()
            .then(|| (code.code.clone(), code.amount_for(subtotal)))
    }

    pub fn lines(v: &Visitor) -> Vec<LineView> {
        let l = v.locale;
        let country = vat_country(v);
        v.cart
            .lines
            .iter()
            .map(|line| LineView {
                variant_id: line.variant_id,
                image: line.image.clone(),
                description: v.shop.line_description(l, line),
                quantity: line.quantity,
                unit_price: v.shop.price(l, line.unit_price),
                total: v.shop.price(l, line.total()),
                total_excl_vat: v
                    .shop
                    .price(l, v.shop.vat.excl_vat(line.category, line.total(), country)),
            })
            .collect()
    }

    /// The rows under the cart: goods before discount, discount, delivery
    /// estimate, total, VAT per rate (not in `compact`), total without VAT.
    pub fn totals(v: &Visitor, compact: bool) -> Vec<TotalRow> {
        totals_with(v, compact, true)
    }

    /// As [`totals`]; `with_estimate: false` leaves out the "Kam vozíme?"
    /// estimate (the checkout sidebar).
    pub fn totals_with(v: &Visitor, compact: bool, with_estimate: bool) -> Vec<TotalRow> {
        let l = v.locale;
        let price = |amount: Czk| v.shop.price(l, amount);
        let discount = applied_discount(v);
        let delivery = v
            .delivery
            .as_ref()
            .filter(|_| with_estimate)
            .map(|d| (d.place.clone(), d.quote(v.shop.czk_per_km)));
        let vat = v.shop.breakdown(
            &v.cart,
            vat_country(v),
            discount.as_ref().map_or(Czk::ZERO, |(_, a)| *a),
            delivery.as_ref().map_or(Czk::ZERO, |(_, q)| q.price()),
        );
        let mut rows = Vec::new();
        let row = |label: String, value: String, kind| TotalRow { label, value, kind };
        if discount.is_some() || delivery.is_some() {
            rows.push(row(
                t::cart::goods_before_discount(l).into(),
                price(v.cart.subtotal()),
                RowKind::Muted,
            ));
        }
        if let Some((code, amount)) = &discount {
            rows.push(row(
                t::cart::discount_line(l, code),
                format!("−{}", price(*amount)),
                RowKind::Discount,
            ));
        }
        if let Some((place, quote)) = &delivery {
            rows.push(row(
                t::delivery_estimate::cart_line(l, quote.round_trip_km(), place),
                v.shop.approx_price(l, quote.price()),
                RowKind::Delivery,
            ));
        }
        rows.push(row(
            t::cart::total_due(l).into(),
            if delivery.is_some() {
                v.shop.approx_price(l, vat.total())
            } else {
                price(vat.total())
            },
            RowKind::Total,
        ));
        if !compact {
            rows.extend(vat_rows(v, &vat.by_rate));
        }
        rows.push(row(
            t::cart::without_vat(l).into(),
            price(vat.total_excl_vat()),
            RowKind::Muted,
        ));
        rows
    }

    /// "DPH 21 %" rows, and their sum when there's more than one rate.
    pub fn vat_rows(v: &Visitor, by_rate: &[(u8, Czk)]) -> Vec<TotalRow> {
        let l = v.locale;
        let mut rows: Vec<TotalRow> = by_rate
            .iter()
            .map(|(rate, amount)| TotalRow {
                label: t::cart::vat_at_rate(l, rate),
                value: v.shop.price(l, *amount),
                kind: RowKind::Muted,
            })
            .collect();
        if by_rate.len() > 1 {
            let sum: Czk = by_rate.iter().map(|(_, a)| *a).sum();
            rows.push(TotalRow {
                label: t::cart::total_vat(l).into(),
                value: v.shop.price(l, sum),
                kind: RowKind::Muted,
            });
        }
        rows
    }

    pub fn summary(v: &Visitor) -> CartSummary {
        let l = v.locale;
        let count = v.cart.total_quantity();
        CartSummary {
            count,
            label: crate::shop::cart_label(l, count),
            lines: lines(v),
            totals: if v.cart.is_empty() {
                Vec::new()
            } else {
                totals(v, true)
            },
            empty_text: t::cart::empty(l).into(),
            view_cart_text: t::cart::view_cart(l).into(),
            view_cart_href: v.link("/kosik"),
        }
    }
}
#[cfg(feature = "ssr")]
pub use build::*;
