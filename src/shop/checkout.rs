//! What the customer entered during checkout (shipping, contact, address),
//! kept in the session until the order is placed.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

const STRING_FIELDS: [&str; 18] = [
    "shipping_method",
    "payment_method",
    "country",
    "first_name",
    "last_name",
    "email",
    "phone",
    "billing_street",
    "billing_city",
    "billing_zip",
    "billing_country",
    "company_name",
    "company_ico",
    "company_dic",
    "delivery_street",
    "delivery_city",
    "delivery_zip",
    "delivery_country",
];
const BOOLEAN_FIELDS: [&str; 3] = ["newsletter_opt_in", "company_purchase", "delivery_address_different"];
const REQUIRED_DETAILS: [&str; 8] = [
    "first_name",
    "last_name",
    "email",
    "phone",
    "billing_street",
    "billing_city",
    "billing_zip",
    "billing_country",
];

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct CheckoutState(BTreeMap<String, String>);

impl CheckoutState {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// A text field; None when empty.
    pub fn get(&self, field: &str) -> Option<&str> {
        self.0
            .get(field)
            .map(String::as_str)
            .filter(|value| !value.trim().is_empty())
    }

    pub fn flag(&self, field: &str) -> bool {
        self.0.get(field).is_some_and(|value| value == "true")
    }

    /// Stores the submitted fields we know; checkboxes become true/false the
    /// way Rails reads them ("0", "false", "off"... are false).
    pub fn update(&mut self, fields: &[(String, String)]) {
        for (name, value) in fields {
            if STRING_FIELDS.contains(&name.as_str()) {
                self.0.insert(name.clone(), value.clone());
            } else if BOOLEAN_FIELDS.contains(&name.as_str()) {
                self.0.insert(name.clone(), boolean(value).to_string());
            }
        }
    }

    pub fn shipping_selected(&self) -> bool {
        self.get("shipping_method").is_some() && self.get("payment_method").is_some()
    }

    pub fn details_complete(&self) -> bool {
        REQUIRED_DETAILS.iter().all(|field| self.get(field).is_some())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn fields(&self) -> &BTreeMap<String, String> {
        &self.0
    }
}

fn boolean(value: &str) -> bool {
    !matches!(value.trim(), "" | "0" | "f" | "F" | "false" | "FALSE" | "off" | "OFF")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
        list.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn keeps_known_fields_and_reads_checkboxes_like_rails() {
        let mut state = CheckoutState::default();
        state.update(&pairs(&[
            ("first_name", "Jana"),
            ("role", "admin"),
            ("company_purchase", "1"),
            ("newsletter_opt_in", "0"),
        ]));
        assert_eq!(state.get("first_name"), Some("Jana"));
        assert_eq!(state.get("role"), None);
        assert!(state.flag("company_purchase"));
        assert!(!state.flag("newsletter_opt_in"));
    }

    #[test]
    fn knows_when_steps_are_done() {
        let mut state = CheckoutState::default();
        assert!(!state.shipping_selected());
        state.update(&pairs(&[("shipping_method", "pickup"), ("payment_method", "cash")]));
        assert!(state.shipping_selected());
        assert!(!state.details_complete());
        state.update(&pairs(&[
            ("first_name", "J"),
            ("last_name", "N"),
            ("email", "j@x.cz"),
            ("phone", "1"),
            ("billing_street", "A 1"),
            ("billing_city", "B"),
            ("billing_zip", "1"),
            ("billing_country", "cz"),
        ]));
        assert!(state.details_complete());
    }
}
