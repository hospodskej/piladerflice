//! The fields of a form, as they came.

use serde::de::DeserializeOwned;

/// A submitted form's fields in their order. A name may repeat (a list of
/// checkboxes, `length[]` fields added by a script).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FormData {
    pairs: Vec<(String, String)>,
}

impl FormData {
    /// Reads an `application/x-www-form-urlencoded` body.
    pub fn parse(body: &[u8]) -> Self {
        Self {
            pairs: form_urlencoded::parse(body).into_owned().collect(),
        }
    }

    pub fn from_pairs(pairs: Vec<(String, String)>) -> Self {
        Self { pairs }
    }

    /// The value of a field; when the name repeats, the last one.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.pairs
            .iter()
            .rev()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// Every value of a repeated field, in order.
    pub fn all(&self, name: &str) -> Vec<&str> {
        self.pairs
            .iter()
            .filter(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
            .collect()
    }

    pub fn pairs(&self) -> &[(String, String)] {
        &self.pairs
    }

    /// The fields as your own type (see [`crate::Form`]).
    pub fn deserialize<T: DeserializeOwned>(&self) -> Result<T, serde_html_form::de::Error> {
        let encoded = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(self.pairs.iter())
            .finish();
        serde_html_form::from_str(&encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_single_and_repeated_fields() {
        let form = FormData::parse(b"name=Jana&name=Eva&length%5B%5D=4+m&length%5B%5D=5+m&empty=");
        assert_eq!(form.get("name"), Some("Eva"));
        assert_eq!(form.all("length[]"), ["4 m", "5 m"]);
        assert_eq!(form.get("empty"), Some(""));
        assert_eq!(form.get("missing"), None);
    }

    #[test]
    fn deserializes_into_a_struct() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct Order {
            name: String,
            #[serde(default)]
            tags: Vec<String>,
            #[serde(default, deserialize_with = "crate::de::checkbox")]
            gift: bool,
        }
        let form = FormData::parse(b"name=Jana&tags=a&tags=b&gift=1");
        assert_eq!(
            form.deserialize::<Order>().unwrap(),
            Order {
                name: "Jana".into(),
                tags: vec!["a".into(), "b".into()],
                gift: true
            }
        );
        let form = FormData::parse(b"name=Eva&csrf_token=x");
        assert!(!form.deserialize::<Order>().unwrap().gift);
    }
}
