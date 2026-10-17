//! Firewood product pages: a grid of containers (lengths across, amounts
//! down) and a row of crates.

use crate::models::catalog::{CatalogVariant, FirewoodGrid};
use crate::ui::eshop::{FirewoodRow, FirewoodVariants, VariantView, VariantViews};

/// `and` is the translated word between amounts ("a" / "und").
pub fn data(
    variants: &[CatalogVariant],
    views: &VariantViews,
    and: &str,
    container: &str,
    crate_image: &str,
) -> FirewoodVariants {
    let in_group = |group: &'static str| {
        variants
            .iter()
            .filter(move |v| v.variant_group.as_deref() == Some(group))
    };

    let grid = FirewoodGrid::new(in_group("kontejner").collect());
    let lengths = grid.lengths();
    let separator = format!(" {and} ");
    let rows: Vec<FirewoodRow> = grid
        .rows()
        .iter()
        .enumerate()
        .map(|(index, amounts)| FirewoodRow {
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
    let crates: Vec<VariantView> = in_group("bedny")
        .map(|v| views.view(v, Some(crate_image.to_string()), String::new()))
        .collect();

    FirewoodVariants {
        lengths: lengths.iter().map(|length| length.to_string()).collect(),
        rows,
        crates,
    }
}
