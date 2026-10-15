//! The cart, in the session. Lines keep the order they were added in.

use serde::{Deserialize, Serialize};

use super::cart_line::{CartLine, Spec};
use super::without_vat;
use crate::models::catalog::{CatalogProduct, CatalogVariant};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct Cart {
    lines: Vec<CartLine>,
}

impl Cart {
    /// Adds `quantity` (at least 1) of a variant; returns the line's id.
    pub fn add(
        &mut self,
        product: &CatalogProduct,
        variant: &CatalogVariant,
        image: Option<String>,
        quantity: i64,
    ) -> String {
        let quantity = quantity.max(1);
        let id = variant.id.to_string();
        match self.lines.iter_mut().find(|line| line.id == id) {
            Some(line) => line.quantity += quantity,
            None => self.lines.push(CartLine {
                id: id.clone(),
                catalog_variant_id: variant.id,
                product_key: product.key.clone(),
                image,
                unit_price_czk: variant.price_czk,
                specs: specs_of(product, variant),
                quantity,
            }),
        }
        id
    }

    /// Sets a line's quantity; 0 or less removes it.
    pub fn update_quantity(&mut self, id: &str, quantity: i64) {
        if quantity <= 0 {
            self.remove(id);
        } else if let Some(line) = self.lines.iter_mut().find(|line| line.id == id) {
            line.quantity = quantity;
        }
    }

    pub fn remove(&mut self, id: &str) {
        self.lines.retain(|line| line.id != id);
    }

    pub fn find(&self, id: &str) -> Option<&CartLine> {
        self.lines.iter().find(|line| line.id == id)
    }

    pub fn lines(&self) -> &[CartLine] {
        &self.lines
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn subtotal_czk(&self) -> i64 {
        self.lines.iter().map(CartLine::line_total_czk).sum()
    }

    pub fn subtotal_excl_vat_czk(&self) -> i64 {
        without_vat(self.subtotal_czk())
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

/// What a variant is, shown after the product name.
fn specs_of(product: &CatalogProduct, variant: &CatalogVariant) -> Vec<Spec> {
    let spec = |kind: &str, value: serde_json::Value| Spec {
        kind: kind.into(),
        value,
        unit_key: None,
        value_de: None,
    };
    let present = |value: &Option<String>| {
        value
            .as_deref()
            .filter(|text| !text.trim().is_empty())
            .map(str::to_string)
    };
    let mut specs = Vec::new();
    match product.template.as_str() {
        "firewood" => {
            let kind = if variant.is_stacked() {
                "common.stacked_variant"
            } else {
                "common.loose_variant"
            };
            specs.push(spec("key", kind.into()));
            if let Some(length) = present(&variant.length_label) {
                specs.push(spec("raw", length.into()));
            }
            if let Some(amount) = variant.amount_value {
                specs.push(Spec {
                    unit_key: Some("common.per_m3".into()),
                    ..spec("amount", amount.into())
                });
            }
        }
        "lumber" => {
            if let Some(grade) = present(&variant.grade) {
                specs.push(Spec {
                    value_de: variant.grade_de.clone(),
                    ..spec("bilingual", grade.into())
                });
            }
            if let Some(length) = present(&variant.length_label) {
                specs.push(spec("raw", length.into()));
            }
            if let (Some(width), Some(height)) = (variant.width_mm, variant.height_mm) {
                specs.push(spec("raw", format!("{width}×{height} mm").into()));
            }
        }
        _ => specs.push(Spec {
            value_de: variant.variant_label_de.clone(),
            ..spec("bilingual", variant.variant_label.clone().unwrap_or_default().into())
        }),
    }
    specs
}
