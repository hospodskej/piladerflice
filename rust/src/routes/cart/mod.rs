//! The cart: the cart page, adding a variant, changing a quantity,
//! removing a line. Changes answer with Turbo Streams (the widget and the
//! cart table update in place) or, without Turbo, a redirect.

mod add;
mod change;
mod show;
mod streams;

pub use add::add;
pub use change::{change, destroy, update};
pub use show::show;
