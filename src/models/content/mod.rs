//! Content edited in the admin or seeded: the weekly promo, services,
//! Sortiment categories, the "might interest you" slides and Google reviews.

mod category_tile;
mod faq_item;
mod promo;
mod review;
mod service;

pub use category_tile::{CategoryTile, category_tiles};
pub use faq_item::{FaqItem, faq_items};
pub use promo::{Promo, weekly_promo};
pub use review::{Review, reviews};
pub use service::{Service, services, services_page};
