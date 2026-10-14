//! The price list tables, from the `pricelist_items` the admin edits.

use std::collections::HashMap;

use minijinja::{Value, context};
use serde::Serialize;

use crate::AppState;
use crate::locale::Locale;
use crate::models::pricelist::{self, PricelistItem, pricelist_html};
use crate::money::Money;

#[derive(Serialize)]
struct PriceRow {
    name: String,
    details: String,
    price: String,
    /// "---" means "no price", shown without the "/ PRM" suffix.
    has_price: bool,
}

#[derive(Serialize)]
struct PriceGroup {
    name: String,
    items: Vec<PriceRow>,
}

/// The template variables of every table: stavebni, palivove_volne,
/// palivove_skladane, zbytky, kamenivo, sluzby_cenik.
pub async fn load(state: &AppState, money: &Money) -> sqlx::Result<Value> {
    let locale = money.locale;
    let mut by_category: HashMap<String, Vec<PricelistItem>> = HashMap::new();
    for item in pricelist::pricelist(&state.db).await? {
        by_category
            .entry(item.category.clone().unwrap_or_default())
            .or_default()
            .push(item);
    }
    let items = |category: &str| by_category.get(category).map(Vec::as_slice).unwrap_or_default();
    let kamenivo = items("kamenivo").first().map(|item| pricelist_html(&item.price(money)));

    Ok(context! {
        stavebni => rows(items("stavebni"), locale, money),
        palivove_volne => groups(items("palivove_volne"), locale, money),
        palivove_skladane => groups(items("palivove_skladane"), locale, money),
        zbytky => rows(items("zbytky"), locale, money),
        kamenivo,
        sluzby_cenik => rows(items("sluzby"), locale, money),
    })
}

fn row(item: &PricelistItem, locale: Locale, money: &Money) -> PriceRow {
    let price = item.price(money);
    PriceRow {
        name: pricelist_html(&item.item_name(locale)),
        details: pricelist_html(&item.details(locale)),
        has_price: price != "---",
        price: pricelist_html(&price),
    }
}

fn rows(items: &[PricelistItem], locale: Locale, money: &Money) -> Vec<PriceRow> {
    items.iter().map(|item| row(item, locale, money)).collect()
}

/// Firewood: grouped by kind of wood (the subcategory), in order of appearance.
fn groups(items: &[PricelistItem], locale: Locale, money: &Money) -> Vec<PriceGroup> {
    let mut groups: Vec<PriceGroup> = Vec::new();
    for item in items {
        let name = pricelist_html(&item.subcategory(locale));
        match groups.iter_mut().find(|group| group.name == name) {
            Some(group) => group.items.push(row(item, locale, money)),
            None => groups.push(PriceGroup {
                name,
                items: vec![row(item, locale, money)],
            }),
        }
    }
    groups
}
