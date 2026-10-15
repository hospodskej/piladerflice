//! What the admin area edits: one file per kind of record, with its
//! columns, its checks and anything special about saving or deleting it.
//! The pages themselves are in `routes/admin/`.
//!
//! - [`record`]: reading a row into a form and writing it back
//! - [`messages`]: the messages of a form that didn't save
//! - [`products`], [`variants`], [`promos`], [`pricelist`], [`options`]:
//!   the editable tables
//! - [`images`]: checking and cleaning uploaded photos
//! - [`ordering`]: moving list items up and down
//! - [`dashboard`]: the numbers on the overview page

pub mod dashboard;
pub mod images;
pub mod messages;
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

    /// Every column is a field of its form (a column left out could never
    /// be saved), apart from those set by the program.
    fn assert_all_in_form(table: &Table, fields: &[&str], set_by_program: &[&str]) {
        for column in table.columns {
            assert!(
                fields.contains(&column.name) || set_by_program.contains(&column.name),
                "{}.{} is not a field of its form",
                table.name,
                column.name
            );
        }
    }

    #[test]
    fn forms_change_every_column() {
        assert_all_in_form(&super::products::TABLE, &super::products::FORM_FIELDS, &[]);
        assert_all_in_form(
            &super::variants::TABLE,
            &super::variants::FORM_FIELDS,
            &["catalog_product_id"],
        );
        assert_all_in_form(&super::promos::TABLE, &super::promos::FORM_FIELDS, &[]);
        assert_all_in_form(&super::pricelist::TABLE, &super::pricelist::FORM_FIELDS, &["position"]);
        assert_all_in_form(&super::options::TABLE, &super::options::FORM_FIELDS, &["position"]);
    }
}
