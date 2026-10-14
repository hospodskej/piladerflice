//! Firewood product pages: a grid of containers (lengths across, amounts
//! down) and a row of crates.

use minijinja::{Value, context};
use serde::Serialize;

use super::variant_view::{VariantView, VariantViews};
use crate::models::catalog::{CatalogVariant, FirewoodGrid};

#[derive(Serialize)]
struct GridRow {
    /// "5", "10", "13 a 14"
    label: String,
    /// One cell per length; empty when there is no such variant.
    cells: Vec<Option<VariantView>>,
}

/// `and` is the translated word between amounts ("a" / "und").
pub fn data(variants: &[CatalogVariant], views: &VariantViews, and: &str, container: &str, crate_image: &str) -> Value {
    let in_group = |group: &'static str| {
        variants
            .iter()
            .filter(move |v| v.variant_group.as_deref() == Some(group))
    };

    let grid = FirewoodGrid::new(in_group("kontejner").collect());
    let lengths = grid.lengths();
    let separator = format!(" {and} ");
    let rows: Vec<GridRow> = grid
        .rows()
        .iter()
        .enumerate()
        .map(|(index, amounts)| GridRow {
            label: amounts.iter().map(i64::to_string).collect::<Vec<_>>().join(&separator),
            cells: lengths
                .iter()
                .map(|length| {
                    grid.variant_at(index, length)
                        .map(|v| views.view(v, Some(container.to_string()), String::new()))
                })
                .collect(),
        })
        .collect();
    let bedny: Vec<VariantView> = in_group("bedny")
        .map(|v| views.view(v, Some(crate_image.to_string()), String::new()))
        .collect();

    context! { lengths, rows, bedny }
}
