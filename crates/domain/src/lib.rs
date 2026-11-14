//! The rules of the Pila Derflice shop, kept apart from the web and the
//! database so they can be tested on their own and shared with the browser.
//!
//! Money is whole Czech crowns ([`Czk`]); the German site shows euros
//! converted at the day's rate ([`money::EurRate`]).

pub mod cart;
pub mod catalog;
pub mod checkout;
pub mod delivery;
pub mod discount;
pub mod money;
pub mod vat;

pub use catalog::Category;
pub use money::Czk;
pub use vat::Country;
