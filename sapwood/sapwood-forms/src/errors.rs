//! What is wrong with a submitted form.

use serde::Serialize;
use serde::ser::SerializeStruct;

/// Messages by field, in the order they were found. In a template:
/// `errors.messages` (all of them), `errors.count`, and `errors.fields.title`
/// (the first message of a field, to show next to it).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Errors {
    list: Vec<(String, String)>,
}

impl Errors {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a message about `field` ("" for the form as a whole).
    pub fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.list.push((field.into(), message.into()));
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Every message, in order.
    pub fn messages(&self) -> Vec<&str> {
        self.list.iter().map(|(_, message)| message.as_str()).collect()
    }

    /// The first message about `field`.
    pub fn on(&self, field: &str) -> Option<&str> {
        self.list
            .iter()
            .find(|(name, _)| name == field)
            .map(|(_, message)| message.as_str())
    }

    pub fn has(&self, field: &str) -> bool {
        self.on(field).is_some()
    }

    /// `Ok(value)` when there are no errors, else the errors.
    pub fn result<T>(self, value: T) -> Result<T, Errors> {
        if self.is_empty() { Ok(value) } else { Err(self) }
    }
}

impl Serialize for Errors {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut fields = std::collections::BTreeMap::new();
        for (field, message) in &self.list {
            fields.entry(field.as_str()).or_insert(message.as_str());
        }
        let mut state = serializer.serialize_struct("Errors", 3)?;
        state.serialize_field("messages", &self.messages())?;
        state.serialize_field("count", &self.len())?;
        state.serialize_field("fields", &fields)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_messages_by_field() {
        let mut errors = Errors::new();
        assert!(errors.clone().result(1).is_ok());
        errors.add("title", "Title is missing");
        errors.add("title", "Title is too short");
        errors.add("price", "Price must be a number");
        assert_eq!(errors.len(), 3);
        assert_eq!(errors.on("title"), Some("Title is missing"));
        assert!(errors.has("price") && !errors.has("key"));

        let json = serde_json::to_value(&errors).unwrap();
        assert_eq!(json["count"], 3);
        assert_eq!(json["fields"]["title"], "Title is missing");
        assert_eq!(json["messages"][2], "Price must be a number");
        assert!(errors.result(()).is_err());
    }
}
