//! Admin form pieces: fields that take their value from the record (or
//! from what was typed, after a refused save), and the error box.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A record's fields as text, as the form shows them.
pub type Values = HashMap<String, String>;

/// What was typed into a form that couldn't be saved, and why.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Refused {
    /// The form's address, so the values go back to the right form.
    pub path: String,
    pub values: Values,
    pub errors: Vec<String>,
}

/// The values a form shows: the refused ones if this form was just
/// refused, else the record's.
pub fn values_for(v: &crate::visitor::Visitor, record: Values) -> (Values, Vec<String>) {
    match &v.admin_refused {
        Some(r) if r.path == v.path => (r.values.clone(), r.errors.clone()),
        _ => (record, Vec::new()),
    }
}

/// "1 chyba", "3 chyby", "5 chyb".
pub fn error_count(n: usize) -> String {
    let word = match n {
        1 => "chyba",
        2..=4 => "chyby",
        _ => "chyb",
    };
    format!("{n} {word}")
}

#[component]
pub fn Errors(errors: Vec<String>) -> impl IntoView {
    (!errors.is_empty()).then(|| {
        view! {
            <div class="admin-flash admin-flash-alert">
                <strong>{error_count(errors.len())}" brání uložení:"</strong>
                <ul>{errors.into_iter().map(|e| view! { <li>{e}</li> }).collect_view()}</ul>
            </div>
        }
    })
}

/// The id and name a field gets in a form for `model` ("catalog_product").
pub fn names(model: &str, field: &str) -> (String, String) {
    (format!("{model}_{field}"), format!("{model}[{field}]"))
}

fn hint_view(hint: Option<&'static str>) -> impl IntoView + use<> {
    hint.map(|h| view! { <p class="admin-form-hint">{h}</p> })
}

pub struct Form<'a> {
    pub model: &'static str,
    pub values: &'a Values,
}

impl Form<'_> {
    pub fn value(&self, field: &str) -> String {
        self.values.get(field).cloned().unwrap_or_default()
    }

    pub fn text(
        &self,
        field: &str,
        label: &'static str,
        hint: Option<&'static str>,
        wide: bool,
    ) -> impl IntoView + use<> {
        self.input("text", field, label, hint, wide, None)
    }

    pub fn number(
        &self,
        field: &str,
        label: &'static str,
        hint: Option<&'static str>,
    ) -> impl IntoView + use<> {
        self.input("number", field, label, hint, false, None)
    }

    pub fn input(
        &self,
        kind: &'static str,
        field: &str,
        label: &'static str,
        hint: Option<&'static str>,
        wide: bool,
        placeholder: Option<&'static str>,
    ) -> impl IntoView + use<> {
        let (id, name) = names(self.model, field);
        let value = self.value(field);
        view! {
            <div class=if wide { "form-group form-group-wide" } else { "form-group" }>
                <label for=id.clone()>{label}</label>
                <input type=kind name=name id=id value=value placeholder=placeholder/>
                {hint_view(hint)}
            </div>
        }
    }

    pub fn textarea(&self, field: &str, label: &'static str, wide: bool) -> impl IntoView + use<> {
        let (id, name) = names(self.model, field);
        let value = self.value(field);
        view! {
            <div class=if wide { "form-group form-group-wide" } else { "form-group" }>
                <label for=id.clone()>{label}</label>
                <textarea name=name id=id>{value}</textarea>
            </div>
        }
    }

    pub fn select(
        &self,
        field: &str,
        label: &'static str,
        options: &[(&'static str, &'static str)],
        hint: Option<&'static str>,
    ) -> impl IntoView + use<> {
        let (id, name) = names(self.model, field);
        let current = self.value(field);
        let options: Vec<_> = options.to_vec();
        view! {
            <div class="form-group">
                <label for=id.clone()>{label}</label>
                <select name=name id=id>
                    {options.into_iter().map(|(text, value)| view! { <option value=value selected=current == value>{text}</option> }).collect_view()}
                </select>
                {hint_view(hint)}
            </div>
        }
    }

    /// A checkbox ("0" posted when unticked, as Rails' check_box did).
    pub fn checkbox(&self, field: &str, label: &'static str) -> impl IntoView + use<> {
        let (id, name) = names(self.model, field);
        let checked = matches!(self.value(field).as_str(), "1" | "true");
        view! {
            <label class="custom-checkbox-label checkout-checkbox">
                <input type="hidden" name=name.clone() value="0"/>
                <input type="checkbox" name=name id=id value="1" checked=checked/>
                <span class="custom-checkbox"></span>
                <span>{label}</span>
            </label>
        }
    }

    /// A photo field: the current picture and a file input that shrinks the
    /// chosen photo in the browser first (attenuator).
    pub fn image(
        &self,
        field: &str,
        label: &'static str,
        current: Option<String>,
        hint: &'static str,
    ) -> impl IntoView + use<> {
        let (id, name) = names(self.model, field);
        view! {
            <div class="form-group">
                <label for=id.clone()>{label}</label>
                {current.map(|src| view! { <img src=src class="admin-image-preview"/> })}
                <crate::admin::ShrinkingFileInput id=id name=name/>
                <p class="admin-form-hint">{hint}</p>
            </div>
        }
    }
}

#[cfg(feature = "ssr")]
pub mod parse {
    //! Reading a posted admin form.

    use super::Values;

    /// The fields of `model` (`model[field]` → field), last value winning
    /// (so a ticked checkbox beats its hidden "0").
    pub fn fields(pairs: &[(String, String)], model: &str) -> Values {
        let prefix = format!("{model}[");
        pairs
            .iter()
            .filter_map(|(k, v)| {
                Some((
                    k.strip_prefix(&prefix)?.strip_suffix(']')?.to_owned(),
                    v.trim().to_owned(),
                ))
            })
            .collect()
    }

    /// A text value, `None` when blank.
    pub fn text(values: &Values, field: &str) -> Option<String> {
        values.get(field).filter(|v| !v.is_empty()).cloned()
    }

    pub fn checked(values: &Values, field: &str) -> bool {
        matches!(values.get(field).map(String::as_str), Some("1" | "true"))
    }

    /// A whole number, `None` when the field is empty; `Err` when it's
    /// there but not a number.
    pub fn int(values: &Values, field: &str) -> Result<Option<i64>, std::num::ParseIntError> {
        values
            .get(field)
            .filter(|v| !v.is_empty())
            .map(|v| v.parse())
            .transpose()
    }

    /// A multipart form: its text fields, and the bytes of file fields.
    pub async fn multipart(
        mut form: axum::extract::Multipart,
    ) -> (Vec<(String, String)>, Vec<(String, Vec<u8>)>) {
        let (mut fields, mut files) = (Vec::new(), Vec::new());
        while let Ok(Some(part)) = form.next_field().await {
            let name = part.name().unwrap_or_default().to_owned();
            if part.file_name().is_some() {
                if let Ok(bytes) = part.bytes().await
                    && !bytes.is_empty()
                {
                    files.push((name, bytes.to_vec()));
                }
            } else if let Ok(text) = part.text().await {
                fields.push((name, text));
            }
        }
        (fields, files)
    }
}
