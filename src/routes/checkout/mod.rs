//! Checkout after the cart page: shipping and payment, contact details,
//! the summary (where the order is placed) and the thank-you page.
//! Every step needs the steps before it (see [`guard`]).

mod confirmation;
mod details;
mod guard;
mod order_draft;
mod shipping;
mod summary;

pub use confirmation::show as confirmation;
pub use details::{show as details, update as update_details};
pub use shipping::{show as shipping, update as update_shipping};
pub use summary::{confirm, show as summary};
