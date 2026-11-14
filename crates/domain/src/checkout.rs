//! What the customer has filled in during checkout so far (kept in the
//! session between the steps).

use crate::vat::Country;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shipping {
    /// Delivery to the customer's address.
    Address,
    /// Pickup at the sawmill.
    Pickup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Payment {
    Cash,
    BankTransfer,
}

impl Shipping {
    pub fn as_str(self) -> &'static str {
        match self {
            Shipping::Address => "address",
            Shipping::Pickup => "pickup",
        }
    }
    pub fn parse(s: &str) -> Option<Shipping> {
        match s {
            "address" => Some(Shipping::Address),
            "pickup" => Some(Shipping::Pickup),
            _ => None,
        }
    }
}

impl Payment {
    pub fn as_str(self) -> &'static str {
        match self {
            Payment::Cash => "cash",
            Payment::BankTransfer => "bank_transfer",
        }
    }
    pub fn parse(s: &str) -> Option<Payment> {
        match s {
            "cash" => Some(Payment::Cash),
            "bank_transfer" => Some(Payment::BankTransfer),
            _ => None,
        }
    }
}

/// A postal address as typed in (any part may still be missing).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostalAddress {
    pub street: Option<String>,
    pub city: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
}

impl PostalAddress {
    pub fn is_complete(&self) -> bool {
        [&self.street, &self.city, &self.zip, &self.country]
            .iter()
            .all(|p| p.as_deref().is_some_and(|v| !v.trim().is_empty()))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckoutState {
    pub shipping: Option<Shipping>,
    pub payment: Option<Payment>,
    /// The delivery country picked on the shipping step.
    pub country: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub newsletter_opt_in: bool,
    pub billing: PostalAddress,
    pub company_purchase: bool,
    pub company_name: Option<String>,
    pub company_ico: Option<String>,
    pub company_dic: Option<String>,
    pub delivery_address_different: bool,
    pub delivery: PostalAddress,
}

/// A form value: trimmed, `None` when blank.
fn value(v: Option<&str>) -> Option<String> {
    v.map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

/// A checkbox: "1" (or "true"/"on") is ticked.
fn ticked(v: Option<&str>) -> bool {
    matches!(v, Some("1" | "true" | "on"))
}

impl CheckoutState {
    /// The shipping step: method, payment, country.
    pub fn update_shipping(&mut self, get: impl Fn(&str) -> Option<String>) {
        self.shipping = get("shipping_method").as_deref().and_then(Shipping::parse);
        self.payment = get("payment_method").as_deref().and_then(Payment::parse);
        self.country = value(get("country").as_deref());
    }

    /// The details step. A checkbox posts "0" (hidden) and then "1" when
    /// ticked, so the caller passes the last value.
    pub fn update_details(&mut self, get: impl Fn(&str) -> Option<String>) {
        let v = |name: &str| value(get(name).as_deref());
        self.first_name = v("first_name");
        self.last_name = v("last_name");
        self.email = v("email");
        self.phone = v("phone");
        self.newsletter_opt_in = ticked(get("newsletter_opt_in").as_deref());
        self.billing = PostalAddress {
            street: v("billing_street"),
            city: v("billing_city"),
            zip: v("billing_zip"),
            country: v("billing_country"),
        };
        self.company_purchase = ticked(get("company_purchase").as_deref());
        self.company_name = v("company_name");
        self.company_ico = v("company_ico");
        self.company_dic = v("company_dic");
        self.delivery_address_different = ticked(get("delivery_address_different").as_deref());
        self.delivery = PostalAddress {
            street: v("delivery_street"),
            city: v("delivery_city"),
            zip: v("delivery_zip"),
            country: v("delivery_country"),
        };
    }

    pub fn shipping_selected(&self) -> bool {
        self.shipping.is_some() && self.payment.is_some()
    }

    pub fn details_complete(&self) -> bool {
        [&self.first_name, &self.last_name, &self.email, &self.phone]
            .iter()
            .all(|p| p.is_some())
            && self.billing.is_complete()
    }

    /// Where the goods go: the delivery address when different, else billing.
    pub fn destination(&self) -> &PostalAddress {
        if self.delivery_address_different {
            &self.delivery
        } else {
            &self.billing
        }
    }

    /// Whose VAT applies, as far as checkout knows: pickup is Czech; for an
    /// address, its country (or the one picked on the shipping step).
    pub fn vat_country(&self) -> Option<Country> {
        match self.shipping? {
            Shipping::Pickup => Some(Country::Cz),
            Shipping::Address => self
                .destination()
                .country
                .as_deref()
                .or(self.country.as_deref())
                .map(|c| Country::or_cz(Some(c))),
        }
    }

    pub fn clear(&mut self) {
        *self = CheckoutState::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |name| {
            pairs
                .iter()
                .rev()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    #[test]
    fn steps() {
        let mut c = CheckoutState::default();
        assert!(!c.shipping_selected());
        c.update_shipping(form(&[
            ("shipping_method", "address"),
            ("payment_method", "cash"),
            ("country", "at"),
        ]));
        assert!(c.shipping_selected());
        assert_eq!(c.vat_country(), Some(Country::At));
        c.update_details(form(&[
            ("first_name", "Jan"),
            ("last_name", "Novák"),
            ("email", "jan@example.cz"),
            ("phone", "1"),
            ("billing_street", "Hlavní 1"),
            ("billing_city", "Znojmo"),
            ("billing_zip", "66902"),
            ("billing_country", "cz"),
            ("company_purchase", "0"),
            ("delivery_address_different", "0"),
            ("delivery_address_different", "1"),
            ("delivery_street", "Wiener 2"),
            ("delivery_city", "Retz"),
            ("delivery_zip", "2070"),
            ("delivery_country", "at"),
        ]));
        assert!(c.details_complete());
        assert!(c.delivery_address_different);
        assert!(!c.company_purchase);
        assert_eq!(c.vat_country(), Some(Country::At));
        c.update_shipping(form(&[
            ("shipping_method", "pickup"),
            ("payment_method", "cash"),
        ]));
        assert_eq!(c.vat_country(), Some(Country::Cz));
    }

    #[test]
    fn blank_is_missing() {
        let mut c = CheckoutState::default();
        c.update_details(form(&[("first_name", "  "), ("billing_street", "x")]));
        assert_eq!(c.first_name, None);
        assert!(!c.details_complete());
    }
}
