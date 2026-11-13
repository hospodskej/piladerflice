//! The totals under a cart: goods, discount, delivery estimate, total, VAT.

use crate::i18n::t;
use crate::visitor::Visitor;
use domain::money::Czk;
use domain::vat::Country;
use leptos::prelude::*;

/// The VAT country before checkout: the estimate's country, else Czech.
pub fn vat_country(v: &Visitor) -> Country {
    v.delivery.as_ref().map_or(Country::Cz, |d| d.country)
}

/// "DPH 21 %" lines, and their sum when the goods have more than one rate.
#[component]
pub fn VatRows(v: Visitor, by_rate: Vec<(u8, Czk)>) -> impl IntoView {
    let l = v.locale;
    let sum: Czk = by_rate.iter().map(|(_, a)| *a).sum();
    let several = by_rate.len() > 1;
    view! {
        {by_rate.into_iter().map(|(rate, amount)| view! {
            <div class="cart-total-row cart-total-excl-vat">
                <span>{t::cart::vat_at_rate(l, rate)}</span>
                <span>{v.shop.price(l, amount)}</span>
            </div>
        }).collect_view()}
        {several.then(|| view! {
            <div class="cart-total-row cart-total-excl-vat">
                <span>{t::cart::total_vat(l)}</span>
                <span>{v.shop.price(l, sum)}</span>
            </div>
        })}
    }
}

#[component]
pub fn CartTotals(
    v: Visitor,
    /// The header dropdown: no VAT lines.
    #[prop(optional)]
    compact: bool,
    /// The discount the cart's code gives now (checked by the caller).
    #[prop(optional)]
    discount: Option<(String, Czk)>,
) -> impl IntoView {
    let l = v.locale;
    let delivery = v.delivery.as_ref().map(|d| (d.place.clone(), d.quote(v.shop.czk_per_km)));
    let discount_czk = discount.as_ref().map_or(Czk::ZERO, |(_, a)| *a);
    let delivery_czk = delivery.as_ref().map_or(Czk::ZERO, |(_, q)| q.price());
    let vat = v.shop.breakdown(&v.cart, vat_country(&v), discount_czk, delivery_czk);
    let approx = if delivery.is_some() { "≈ " } else { "" };
    view! {
        {(discount.is_some() || delivery.is_some()).then(|| view! {
            <div class="cart-total-row cart-total-excl-vat">
                <span>{t::cart::goods_before_discount(l)}</span>
                <span>{v.shop.price(l, v.cart.subtotal())}</span>
            </div>
        })}
        {discount.map(|(code, amount)| view! {
            <div class="cart-total-row cart-total-discount">
                <span>{t::cart::discount_line(l, code)}</span>
                <span>"−"{v.shop.price(l, amount)}</span>
            </div>
        })}
        {delivery.map(|(place, quote)| view! {
            <div class="cart-total-row cart-total-excl-vat cart-total-delivery">
                <span>{t::delivery_estimate::cart_line(l, quote.round_trip_km(), place)}</span>
                <span>"≈ "{v.shop.price(l, quote.price())}</span>
            </div>
        })}
        <div class="cart-total-row">
            <span>{t::cart::total_due(l)}</span>
            <strong>{approx}{v.shop.price(l, vat.total())}</strong>
        </div>
        {(!compact).then(|| view! { <VatRows v=v.clone() by_rate=vat.by_rate.clone()/> })}
        <div class="cart-total-row cart-total-excl-vat">
            <span>{t::cart::without_vat(l)}</span>
            <span>{v.shop.price(l, vat.total_excl_vat())}</span>
        </div>
    }
}
