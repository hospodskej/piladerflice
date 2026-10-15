//! The site's translations: `locales/cs.yml` and `locales/de.yml`, read with
//! sapwood-i18n. A text the German file doesn't have yet falls back to Czech.

use std::path::Path;

pub use sapwood_i18n::{Translations, interpolate, missing};

use crate::locale::Locale;

pub fn load(dir: &Path) -> anyhow::Result<Translations> {
    Ok(Translations::load_dir(dir)?.with_fallback(Locale::Cs.code()))
}
