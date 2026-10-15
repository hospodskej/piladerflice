//! Shopping: the cart and checkout details (both kept in the session) and
//! the delivery price.

pub mod cart;
pub mod cart_image;
pub mod cart_line;
pub mod checkout;
pub mod delivery;
pub mod describe;

/// Czech VAT, included in every price.
pub const VAT_RATE: f64 = 0.21;

/// The price without VAT, rounded to whole crowns.
pub fn without_vat(czk: i64) -> i64 {
    (czk as f64 / (1.0 + VAT_RATE)).round() as i64
}

/// The VAT inside a price incl. VAT.
pub fn vat_in(czk: i64) -> i64 {
    czk - without_vat(czk)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vat_is_21_percent_of_the_net_price() {
        assert_eq!(without_vat(21500), 17769);
        assert_eq!(vat_in(21500), 3731);
        assert_eq!(without_vat(121), 100);
    }
}
