//! What the admin area edits: one file per kind of record, with its
//! columns, its checks (Rails' validations, with Rails' messages) and
//! anything special about saving or deleting it. The pages themselves are
//! in `routes/admin/`.
//!
//! - [`record`]: reading a row into a form and writing it back
//! - [`products`], [`variants`], [`promos`], [`pricelist`], [`options`]:
//!   the editable tables
//! - [`images`]: checking and cleaning uploaded photos
//! - [`ordering`]: moving list items up and down
//! - [`dashboard`]: the numbers on the overview page

pub mod dashboard;
pub mod images;
pub mod options;
pub mod ordering;
pub mod pricelist;
pub mod products;
pub mod promos;
pub mod record;
pub mod variants;

#[cfg(test)]
mod tests {
    use super::record::Table;

    /// Every column the admin edits is one its form may change (a column
    /// left out could never be saved), apart from those set by the program.
    fn assert_all_permitted(table: &Table, permitted: &[&str], set_by_program: &[&str]) {
        for column in table.columns {
            assert!(
                permitted.contains(&column.name) || set_by_program.contains(&column.name),
                "{}.{} is not in the permitted fields",
                table.name,
                column.name
            );
        }
    }

    #[test]
    fn forms_may_change_every_column() {
        assert_all_permitted(&super::products::TABLE, &super::products::PERMITTED, &[]);
        assert_all_permitted(
            &super::variants::TABLE,
            &super::variants::PERMITTED,
            &["catalog_product_id"],
        );
        assert_all_permitted(&super::promos::TABLE, &super::promos::PERMITTED, &[]);
        assert_all_permitted(&super::pricelist::TABLE, &super::pricelist::PERMITTED, &["position"]);
        assert_all_permitted(&super::options::TABLE, &super::options::PERMITTED, &["position"]);
    }
}
