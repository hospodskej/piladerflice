//! One line of the cart: a variant and how many. The product's name, price
//! and specs are copied in when it is added, so the cart (and later the
//! order) keeps what the customer saw.

use serde::{Deserialize, Serialize};

/// A property shown after the product name: "Skládané", "100 cm", "13 PRM"...
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Spec {
    /// "key" (a translation key), "amount" (number + unit), "bilingual"
    /// (Czech/German text) or "raw" (text as is).
    #[serde(rename = "type")]
    pub kind: String,
    pub value: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_de: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CartLine {
    /// The line's id: the variant id as text.
    pub id: String,
    pub catalog_variant_id: i64,
    pub product_key: String,
    pub image: Option<String>,
    pub unit_price_czk: i64,
    pub specs: Vec<Spec>,
    pub quantity: i64,
}

impl CartLine {
    pub fn line_total_czk(&self) -> i64 {
        self.unit_price_czk * self.quantity
    }
}

/// A line as an order keeps it (the order's `items_snapshot` is a JSON list
/// of these): what was bought, at what price.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OrderedItem {
    pub catalog_variant_id: i64,
    pub product_key: String,
    pub image: Option<String>,
    pub unit_price_czk: i64,
    #[serde(default)]
    pub specs: Vec<Spec>,
    pub quantity: i64,
}

impl From<&CartLine> for OrderedItem {
    fn from(line: &CartLine) -> Self {
        Self {
            catalog_variant_id: line.catalog_variant_id,
            product_key: line.product_key.clone(),
            image: line.image.clone(),
            unit_price_czk: line.unit_price_czk,
            specs: line.specs.clone(),
            quantity: line.quantity,
        }
    }
}

impl From<OrderedItem> for CartLine {
    fn from(item: OrderedItem) -> Self {
        Self {
            id: item.catalog_variant_id.to_string(),
            catalog_variant_id: item.catalog_variant_id,
            product_key: item.product_key,
            image: item.image,
            unit_price_czk: item.unit_price_czk,
            specs: item.specs,
            quantity: item.quantity,
        }
    }
}
