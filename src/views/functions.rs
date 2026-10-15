//! The functions templates can call besides `t(...)` (sapwood-i18n) and
//! `asset(...)` (the asset index, see `templates.rs`): `path(...)`.

use minijinja::{Error, State};

use crate::locale::{self, Locale};

/// The language of the page being rendered (the `locale` variable).
pub fn current_locale(state: &State) -> Locale {
    state
        .lookup("locale")
        .and_then(|value| value.as_str().and_then(Locale::from_code))
        .unwrap_or(Locale::Cs)
}

/// `path("/kontakt#cenik")`: a site link in the page's language.
pub fn path(state: &State, path: String) -> Result<String, Error> {
    Ok(locale::localized_path(&path, current_locale(state)))
}
