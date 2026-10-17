//! The e-shop: the product grid with its filters, and the product pages.
//! A product page has one of three layouts (`template` in the database),
//! each in its own file.

mod firewood;
mod listing;
mod lumber;
mod product;
mod recommendations;
mod simple_variant;

pub use listing::index;
pub use product::show;
