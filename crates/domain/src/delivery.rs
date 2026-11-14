//! Delivery to an address: a price per km, there and back, one trip per order.

use crate::money::Czk;
use serde::{Deserialize, Serialize};

/// The admin's price per km when none is saved.
pub const DEFAULT_CZK_PER_KM: i64 = 40;
/// The highest price per km the admin may set.
pub const MAX_CZK_PER_KM: i64 = 1000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryQuote {
    pub one_way_km: i64,
    pub czk_per_km: i64,
}

impl DeliveryQuote {
    pub fn new(one_way_km: i64, czk_per_km: i64) -> DeliveryQuote {
        DeliveryQuote {
            one_way_km: one_way_km.max(0),
            czk_per_km,
        }
    }

    pub fn round_trip_km(self) -> i64 {
        self.one_way_km * 2
    }

    pub fn price(self) -> Czk {
        Czk(self.round_trip_km() * self.czk_per_km)
    }
}

/// A delivery estimate for a place, kept in the visitor's session. Only the
/// distance is kept, so a new price per km applies straight away.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryEstimate {
    pub query: String,
    pub place: String,
    pub zip: Option<String>,
    pub country: crate::Country,
    pub one_way_km: i64,
}

impl DeliveryEstimate {
    pub fn quote(&self, czk_per_km: i64) -> DeliveryQuote {
        DeliveryQuote::new(self.one_way_km, czk_per_km)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_and_back() {
        let q = DeliveryQuote::new(23, 40);
        assert_eq!(q.round_trip_km(), 46);
        assert_eq!(q.price(), Czk(1840));
    }
}
