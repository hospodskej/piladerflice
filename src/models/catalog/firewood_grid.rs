//! The container grid on a firewood page.

use super::CatalogVariant;

/// The container grid on a firewood page: lengths across, amounts down.
///
/// Loose wood (25, 33 cm) and stacked wood (50 cm, 1 m) have different top
/// amounts (14 and 13 PRM), so rows are matched by position, not by value:
/// the n-th amount of each kind of wood shares a row, labelled with its
/// amounts ("5", "10", "13 a 14").
pub struct FirewoodGrid<'a> {
    variants: Vec<&'a CatalogVariant>,
    /// (stacked?, its amounts ascending), in order of first appearance.
    amounts_by_kind: Vec<(bool, Vec<i64>)>,
}

impl<'a> FirewoodGrid<'a> {
    pub fn new(variants: Vec<&'a CatalogVariant>) -> Self {
        let mut amounts_by_kind: Vec<(bool, Vec<i64>)> = Vec::new();
        for variant in &variants {
            let kind = variant.is_stacked();
            let index = match amounts_by_kind.iter().position(|(k, _)| *k == kind) {
                Some(index) => index,
                None => {
                    amounts_by_kind.push((kind, Vec::new()));
                    amounts_by_kind.len() - 1
                }
            };
            if let Some(amount) = variant.amount_value
                && !amounts_by_kind[index].1.contains(&amount)
            {
                amounts_by_kind[index].1.push(amount);
            }
        }
        for (_, amounts) in &mut amounts_by_kind {
            amounts.sort_unstable();
        }
        Self {
            variants,
            amounts_by_kind,
        }
    }

    pub fn lengths(&self) -> Vec<String> {
        let mut lengths: Vec<String> = Vec::new();
        for variant in &self.variants {
            let length = variant.length_label.clone().unwrap_or_default();
            if !lengths.contains(&length) {
                lengths.push(length);
            }
        }
        lengths
    }

    /// One entry per row: the amounts in it, ascending, e.g. [13, 14].
    pub fn rows(&self) -> Vec<Vec<i64>> {
        let row_count = self
            .amounts_by_kind
            .iter()
            .map(|(_, amounts)| amounts.len())
            .max()
            .unwrap_or(0);
        (0..row_count)
            .map(|index| {
                let mut row: Vec<i64> = self
                    .amounts_by_kind
                    .iter()
                    .filter_map(|(_, amounts)| amounts.get(index).copied())
                    .collect();
                row.sort_unstable();
                row.dedup();
                row
            })
            .collect()
    }

    /// The variant of this length in the given row, if there is one.
    pub fn variant_at(&self, row_index: usize, length: &str) -> Option<&'a CatalogVariant> {
        self.variants.iter().copied().find(|variant| {
            variant.length_label.as_deref().unwrap_or_default() == length
                && self
                    .amounts_by_kind
                    .iter()
                    .find(|(kind, _)| *kind == variant.is_stacked())
                    .and_then(|(_, amounts)| amounts.get(row_index))
                    .copied()
                    == variant.amount_value
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_data::variant;
    use super::*;

    #[test]
    fn loose_and_stacked_top_amounts_share_the_last_row() {
        let variants = [
            variant("25 cm", 5, false, 1, true),
            variant("25 cm", 14, false, 1, true),
            variant("50 cm", 5, true, 1, true),
            variant("50 cm", 13, true, 1, true),
        ];
        let grid = FirewoodGrid::new(variants.iter().collect());
        assert_eq!(grid.lengths(), ["25 cm", "50 cm"]);
        assert_eq!(grid.rows(), vec![vec![5], vec![13, 14]]);
        assert_eq!(grid.variant_at(1, "25 cm").and_then(|v| v.amount_value), Some(14));
        assert_eq!(grid.variant_at(1, "50 cm").and_then(|v| v.amount_value), Some(13));
        assert!(grid.variant_at(2, "50 cm").is_none());
    }
}
