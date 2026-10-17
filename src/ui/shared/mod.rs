//! Pieces shown on several pages.

pub mod category_tile;
pub mod custom_demand;
pub mod how_to_order;
pub mod photo_carousel;
pub mod product_card;
pub mod wood_types;

pub use category_tile::CategoryTile;
pub use custom_demand::CustomDemand;
pub use how_to_order::HowToOrder;
pub use photo_carousel::PhotoCarousel;
pub use product_card::{ProductCard, ProductCardView};
pub use wood_types::WoodTypes;
