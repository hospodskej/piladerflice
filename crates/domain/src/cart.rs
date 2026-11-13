//! The shopping cart, kept in the visitor's session until the order is placed.
//! Each line remembers the price and description it was added with.

use crate::catalog::Category;
use crate::money::Czk;
use serde::{Deserialize, Serialize};

/// One part of a cart line's description ("Volně ložené", "1 m", "4 m³"...).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Spec {
    /// A translation key, e.g. `common.stacked_variant`.
    Key { value: String },
    /// Text that's the same in every language (a length, dimensions).
    Raw { value: String },
    /// A number followed by a translated unit (`unit_key`).
    Amount { value: i64, unit_key: String },
    /// Text the admin wrote in Czech and (optionally) German.
    Bilingual { value: String, value_de: Option<String> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CartLine {
    pub variant_id: i64,
    pub product_key: String,
    pub image: Option<String>,
    pub category: Category,
    /// The customer price (with VAT) at the time it was added.
    pub unit_price: Czk,
    pub specs: Vec<Spec>,
    pub quantity: u32,
}

impl CartLine {
    pub fn total(&self) -> Czk {
        self.unit_price * i64::from(self.quantity)
    }
}

/// The most of one variant a cart can hold.
pub const MAX_QUANTITY: u32 = 999;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cart {
    pub lines: Vec<CartLine>,
    /// The code the customer entered; checked again on every use.
    pub discount_code: Option<String>,
}

impl Cart {
    /// Adds `quantity` (at least 1) of a variant; merges with a line already
    /// in the cart.
    pub fn add(&mut self, line: CartLine) {
        let quantity = line.quantity.clamp(1, MAX_QUANTITY);
        match self.lines.iter_mut().find(|l| l.variant_id == line.variant_id) {
            Some(existing) => existing.quantity = (existing.quantity + quantity).min(MAX_QUANTITY),
            None => self.lines.push(CartLine { quantity, ..line }),
        }
    }

    /// Sets the quantity of a line; zero removes it.
    pub fn set_quantity(&mut self, variant_id: i64, quantity: u32) {
        if quantity == 0 {
            self.remove(variant_id);
        } else if let Some(line) = self.lines.iter_mut().find(|l| l.variant_id == variant_id) {
            line.quantity = quantity.min(MAX_QUANTITY);
        }
    }

    pub fn remove(&mut self, variant_id: i64) {
        self.lines.retain(|l| l.variant_id != variant_id);
    }

    pub fn line(&self, variant_id: i64) -> Option<&CartLine> {
        self.lines.iter().find(|l| l.variant_id == variant_id)
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn total_quantity(&self) -> u32 {
        self.lines.iter().map(|l| l.quantity).sum()
    }

    pub fn subtotal(&self) -> Czk {
        self.lines.iter().map(CartLine::total).sum()
    }

    /// (category, line total) for [`crate::vat::VatBreakdown`].
    pub fn vat_lines(&self) -> impl Iterator<Item = (Category, Czk)> + '_ {
        self.lines.iter().map(|l| (l.category, l.total()))
    }

    pub fn clear(&mut self) {
        *self = Cart::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(variant_id: i64, price: i64, quantity: u32) -> CartLine {
        CartLine {
            variant_id,
            product_key: "buk".into(),
            image: None,
            category: Category::Firewood,
            unit_price: Czk(price),
            specs: vec![Spec::Raw { value: "33 cm".into() }],
            quantity,
        }
    }

    #[test]
    fn adds_and_merges() {
        let mut cart = Cart::default();
        cart.add(line(1, 1900, 2));
        cart.add(line(1, 1900, 1));
        cart.add(line(2, 1000, 0)); // at least one
        assert_eq!(cart.lines.len(), 2);
        assert_eq!(cart.total_quantity(), 4);
        assert_eq!(cart.subtotal(), Czk(1900 * 3 + 1000));
    }

    #[test]
    fn zero_quantity_removes() {
        let mut cart = Cart::default();
        cart.add(line(1, 100, 1));
        cart.set_quantity(1, 0);
        assert!(cart.is_empty());
    }

    #[test]
    fn spec_json_matches_old_snapshots() {
        let spec = Spec::Amount { value: 4, unit_key: "common.per_m3".into() };
        assert_eq!(serde_json::to_string(&spec).unwrap(), r#"{"type":"amount","value":4,"unit_key":"common.per_m3"}"#);
    }
}
