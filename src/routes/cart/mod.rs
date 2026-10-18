//! The cart: the cart page, adding a variant, changing a quantity,
//! removing a line. These are the plain forms, which redirect; the page's
//! islands make the same changes through server functions.

mod add;
mod change;
mod show;

pub use add::add;
pub use change::{destroy, update};
pub use show::show;
