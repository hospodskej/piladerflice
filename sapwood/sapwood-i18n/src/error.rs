//! What can go wrong loading translations.

use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    /// A translation file couldn't be read.
    Read { file: PathBuf, source: std::io::Error },
    /// A translation file isn't valid YAML (`locale` names the language).
    Parse { locale: String, message: String },
    /// A translation file's top level isn't a map of keys.
    NotAMap { locale: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Read { file, source } => write!(f, "reading {}: {source}", file.display()),
            Error::Parse { locale, message } => write!(f, "parsing the {locale} translations: {message}"),
            Error::NotAMap { locale } => write!(f, "the {locale} translations must be a map of keys"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Read { source, .. } => Some(source),
            _ => None,
        }
    }
}
