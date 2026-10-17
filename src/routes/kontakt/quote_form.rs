//! The item rows of the quote request form: one section per category, its
//! choices from the `inquiry_form_options` the admin edits.

use crate::AppState;
use crate::locale::Locale;
use crate::models::inquiry_options::{self, INQUIRY_CATEGORIES};
use crate::ui::contact::{FormField, FormSection};

pub async fn sections(state: &AppState, locale: Locale) -> sqlx::Result<Vec<FormSection>> {
    let options = inquiry_options::inquiry_options(&state.db).await?;
    Ok(INQUIRY_CATEGORIES
        .iter()
        .map(|category| FormSection {
            category,
            fields: inquiry_options::inquiry_row_fields(category)
                .iter()
                .map(|field| FormField {
                    name: field,
                    options: options
                        .iter()
                        .filter(|option| option.category == *category && option.field == *field)
                        .map(|option| option.value(locale))
                        .collect(),
                    custom: *category == "stavebni" || *field == "mnozstvi",
                    number: *category == "stavebni" && *field == "mnozstvi",
                })
                .collect(),
        })
        .collect())
}
