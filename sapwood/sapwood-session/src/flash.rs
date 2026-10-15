//! A message for the next page ("Order placed", "Wrong password"), shown
//! once.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "text", rename_all = "lowercase")]
pub enum Flash {
    /// Something went well.
    Notice(String),
    /// Something went wrong.
    Alert(String),
}
