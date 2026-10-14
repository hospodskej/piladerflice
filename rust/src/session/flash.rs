//! A message for the next page ("Poptávka odeslána", "Přihlášení se
//! nezdařilo"...), shown once at the top, like Rails' flash.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "text", rename_all = "lowercase")]
pub enum Flash {
    /// Something went well (green).
    Notice(String),
    /// Something went wrong (red).
    Alert(String),
}
