//! Reading HTML form values into your types with serde.

use serde::{Deserialize, Deserializer};

/// A checkbox: checked sends a value ("1", "on", "true"...), unchecked
/// sends nothing. Use with `#[serde(default, deserialize_with = ...)]`, so
/// a missing field is `false`. "0", "false", "off", "no" and "" are false
/// too, for forms that send a hidden "0" before the box.
pub fn checkbox<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    let values = OneOrMany::deserialize(deserializer)?.into_vec();
    Ok(values.last().is_some_and(|value| is_true(value)))
}

/// Whether a submitted value means "yes".
pub fn is_true(value: &str) -> bool {
    !matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "0" | "f" | "false" | "off" | "no" | "n"
    )
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    fn into_vec(self) -> Vec<String> {
        match self {
            OneOrMany::One(value) => vec![value],
            OneOrMany::Many(values) => values,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_that_mean_yes() {
        for yes in ["1", "on", "true", "yes", "TRUE"] {
            assert!(is_true(yes), "{yes}");
        }
        for no in ["", "0", "off", "false", "No", " 0 "] {
            assert!(!is_true(no), "{no}");
        }
    }
}
