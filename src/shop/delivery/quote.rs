//! The delivery price: the truck drives there and back at 40 Kč per
//! kilometre, one trip whatever the amount (incl. VAT, like the goods).
//! Shown as an approximate price.
//!
//! Example from Štěpán: 50 km away -> 2 × 50 km × 40 Kč = 4 000 Kč.

use super::route_service::RouteService;
use crate::shop::checkout::CheckoutState;

pub const CZK_PER_KM: i64 = 40;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeliveryQuote {
    pub one_way_km: i64,
}

impl DeliveryQuote {
    pub fn round_trip_km(&self) -> i64 {
        self.one_way_km * 2
    }

    pub fn price_czk(&self) -> i64 {
        self.round_trip_km() * CZK_PER_KM
    }

    /// The quote for the checkout's address; None for pickup or when the
    /// distance can't be found (then the price is agreed individually).
    pub async fn for_checkout(checkout: &CheckoutState, routes: &RouteService) -> Option<DeliveryQuote> {
        if checkout.get("shipping_method") != Some("address") {
            return None;
        }
        let prefix = if checkout.flag("delivery_address_different") {
            "delivery"
        } else {
            "billing"
        };
        let field = |name: &str| {
            checkout
                .get(&format!("{prefix}_{name}"))
                .unwrap_or_default()
                .to_string()
        };
        let km = routes
            .one_way_km(&field("street"), &field("city"), &field("zip"), &field("country"))
            .await?;
        Some(DeliveryQuote { one_way_km: km })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepans_example() {
        let quote = DeliveryQuote { one_way_km: 50 };
        assert_eq!(quote.round_trip_km(), 100);
        assert_eq!(quote.price_czk(), 4000);
    }
}
