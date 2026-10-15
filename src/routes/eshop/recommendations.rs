//! "You might also like": four other products, picked at random.

use rand::seq::SliceRandom;

use super::super::Result;
use super::super::components::ProductCard;
use crate::AppState;
use crate::models::catalog::{self, CatalogProduct};
use crate::models::upload::{self, Owner};
use crate::money::Money;

pub async fn for_product(state: &AppState, current: &CatalogProduct, money: &Money) -> Result<Vec<ProductCard>> {
    let mut others: Vec<CatalogProduct> = catalog::active_products(&state.db)
        .await?
        .into_iter()
        .filter(|p| p.key != current.key)
        .collect();
    others.shuffle(&mut rand::rng());
    others.truncate(4);

    let starting_prices = catalog::starting_prices(&state.db).await?;
    let images = upload::images_of(&state.db, Owner::Product).await?;
    let price_from = state.translations.text(money.locale, "common.price_from", &[]);
    Ok(others
        .iter()
        .map(|product| ProductCard::new(product, &starting_prices, &images, money, &price_from))
        .collect())
}
