//! How often the same address (or account) may try something.

use refractory::Rule;
use std::time::Duration;

const fn minutes(n: u64) -> Duration {
    Duration::from_secs(n * 60)
}

pub const LOGIN: Rule = Rule::new("login", 10, minutes(3));
/// Against one account from any address.
pub const LOGIN_PER_ACCOUNT: Rule = Rule::new("login-per-account", 20, minutes(60));
pub const REGISTRATION: Rule = Rule::new("registration", 5, minutes(60));
pub const DISCOUNT_CODE: Rule = Rule::new("discount-code", 10, minutes(10));
pub const DELIVERY_LOOKUP: Rule = Rule::new("delivery-lookup", 20, minutes(10));
pub const INQUIRY: Rule = Rule::new("inquiry", 5, minutes(10));
pub const ORDER: Rule = Rule::new("order", 5, minutes(10));
pub const PASSWORD_RESET: Rule = Rule::new("password-reset", 5, minutes(10));
pub const PASSWORD_UPDATE: Rule = Rule::new("password-update", 10, minutes(10));
pub const AVATAR: Rule = Rule::new("avatar", 10, minutes(10));
pub const ADDRESS_SUGGESTIONS: Rule = Rule::new("address-suggestions", 120, minutes(1));
