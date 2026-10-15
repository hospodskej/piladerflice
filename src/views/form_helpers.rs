//! Form fields for the admin templates.
//!
//! - fields of a record's form: `text_field(f, "title")`, `number_field`,
//!   `text_area`, `select(f, "template", options)`, `check_box`,
//!   `file_field`. `f` is the form ([`FormModel`]): a field's name and id
//!   are the column's name, its value the record's, and a field with an
//!   error gets the class `field-error`.
//! - loose fields, tied to a form elsewhere on the page with `form=`:
//!   `text_input(name, value)`, `hidden_input(name, value)`
//! - buttons: `submit(label, class=...)`, `action_button(label, url)` (a
//!   form with one button that posts to `url`, e.g. "Smazat"), and
//!   `csrf_field()` (the form protection field every form needs)

use minijinja::value::{Kwargs, Value};
use minijinja::{Environment, Error, State};
use serde::Serialize;

use super::text::{error_count, escape};
use crate::admin::record::Values;
use crate::forms::Errors;

/// A record's form as the templates get it.
#[derive(Debug, Serialize)]
pub struct FormModel {
    pub values: Values,
    /// Fields with an error.
    pub error_fields: Vec<String>,
    /// The messages, for the box above the form.
    pub errors: Vec<String>,
    /// "2 chyby"
    pub error_count: String,
}

impl FormModel {
    pub fn new(values: Values, errors: &Errors) -> Self {
        let error_fields = values
            .names()
            .filter(|name| errors.has(name))
            .map(str::to_string)
            .collect();
        Self {
            values,
            error_fields,
            errors: errors.messages().into_iter().map(str::to_string).collect(),
            error_count: error_count(errors.len()),
        }
    }
}

pub fn register(env: &mut Environment<'static>) {
    env.add_function("text_field", |f: Value, name: String, kwargs: Kwargs| {
        input(&f, &name, "text", kwargs)
    });
    env.add_function("number_field", |f: Value, name: String, kwargs: Kwargs| {
        input(&f, &name, "number", kwargs)
    });
    env.add_function("text_area", text_area);
    env.add_function("select", select);
    env.add_function("check_box", check_box);
    env.add_function("file_field", file_field);
    env.add_function("text_input", text_input);
    env.add_function("hidden_input", hidden_input);
    env.add_function("submit", submit);
    env.add_function("action_button", action_button);
    env.add_function("csrf_field", csrf_field);
}

/// The parts of `f` a field needs.
struct Field {
    name: String,
    value: Option<String>,
    has_error: bool,
}

fn field(f: &Value, name: &str) -> Result<Field, Error> {
    let value = f.get_attr("values")?.get_item(&Value::from(name))?;
    let value = (!value.is_none() && !value.is_undefined()).then(|| value.to_string());
    let has_error = f
        .get_attr("error_fields")?
        .try_iter()
        .map(|mut fields| fields.any(|field| field.as_str() == Some(name)))
        .unwrap_or(false);
    Ok(Field {
        name: name.to_string(),
        value,
        has_error,
    })
}

impl Field {
    /// name, id and, with an error, the class that frames it red.
    fn attributes(&self) -> String {
        attributes(&[
            ("name", Some(&self.name)),
            ("id", Some(&self.name)),
            ("class", self.has_error.then_some("field-error")),
        ])
    }
}

/// ` name="value"` for each given attribute, escaped.
fn attributes(pairs: &[(&str, Option<&str>)]) -> String {
    pairs
        .iter()
        .filter_map(|(name, value)| value.map(|value| format!(r#" {name}="{}""#, escape(value))))
        .collect()
}

fn optional(kwargs: &Kwargs, name: &str) -> Result<Option<String>, Error> {
    kwargs.get::<Option<String>>(name)
}

fn input(f: &Value, name: &str, kind: &str, kwargs: Kwargs) -> Result<Value, Error> {
    let field = field(f, name)?;
    let placeholder = optional(&kwargs, "placeholder")?;
    kwargs.assert_all_used()?;
    Ok(Value::from_safe_string(format!(
        "<input type=\"{kind}\"{}{}>",
        field.attributes(),
        attributes(&[
            ("value", field.value.as_deref()),
            ("placeholder", placeholder.as_deref()),
        ])
    )))
}

fn text_area(f: Value, name: String) -> Result<Value, Error> {
    let field = field(&f, &name)?;
    Ok(Value::from_safe_string(format!(
        "<textarea{}>{}</textarea>",
        field.attributes(),
        escape(field.value.as_deref().unwrap_or_default())
    )))
}

/// `options` is a list of [label, value]; a none value is the empty choice.
fn select(f: Value, name: String, options: Value) -> Result<Value, Error> {
    let field = field(&f, &name)?;
    let current = field.value.clone().unwrap_or_default();
    let mut html = format!("<select{}>", field.attributes());
    for option in options.try_iter()? {
        let label = option.get_item(&Value::from(0))?.to_string();
        let value = option.get_item(&Value::from(1))?;
        let value = if value.is_none() {
            String::new()
        } else {
            value.to_string()
        };
        html.push_str(&format!(
            "<option{}>{}</option>",
            attributes(&[
                ("value", Some(&value)),
                ("selected", (value == current).then_some("selected")),
            ]),
            escape(&label)
        ));
    }
    html.push_str("</select>");
    Ok(Value::from_safe_string(html))
}

/// An unticked box sends nothing; the record reads that as false.
fn check_box(f: Value, name: String) -> Result<Value, Error> {
    let field = field(&f, &name)?;
    let checked = field.value.as_deref().is_some_and(sapwood_forms::de::is_true);
    Ok(Value::from_safe_string(format!(
        "<input type=\"checkbox\" value=\"1\"{}{}>",
        field.attributes(),
        attributes(&[("checked", checked.then_some("checked"))])
    )))
}

fn file_field(f: Value, name: String, kwargs: Kwargs) -> Result<Value, Error> {
    let field = field(&f, &name)?;
    let accept = optional(&kwargs, "accept")?;
    kwargs.assert_all_used()?;
    Ok(Value::from_safe_string(format!(
        "<input type=\"file\"{}{}>",
        field.attributes(),
        attributes(&[("accept", accept.as_deref())])
    )))
}

/// A field outside a record's form, tied to a form elsewhere on the page
/// with `form=`. No value when there is none.
fn text_input(name: String, value: Option<Value>, kwargs: Kwargs) -> Result<Value, Error> {
    let value = value
        .filter(|value| !value.is_none() && !value.is_undefined())
        .map(|value| value.to_string());
    let (placeholder, form, aria_label) = (
        optional(&kwargs, "placeholder")?,
        optional(&kwargs, "form")?,
        optional(&kwargs, "aria_label")?,
    );
    kwargs.assert_all_used()?;
    Ok(Value::from_safe_string(format!(
        "<input type=\"text\"{}>",
        attributes(&[
            ("name", Some(&name)),
            ("value", value.as_deref()),
            ("placeholder", placeholder.as_deref()),
            ("form", form.as_deref()),
            ("aria-label", aria_label.as_deref()),
        ])
    )))
}

fn hidden_input(name: String, value: String, kwargs: Kwargs) -> Result<Value, Error> {
    let form = optional(&kwargs, "form")?;
    kwargs.assert_all_used()?;
    Ok(Value::from_safe_string(format!(
        "<input type=\"hidden\"{}>",
        attributes(&[
            ("name", Some(&name)),
            ("value", Some(&value)),
            ("form", form.as_deref()),
        ])
    )))
}

fn submit(label: String, kwargs: Kwargs) -> Result<Value, Error> {
    let (class, form) = (optional(&kwargs, "class")?, optional(&kwargs, "form")?);
    kwargs.assert_all_used()?;
    Ok(Value::from_safe_string(format!(
        "<button type=\"submit\"{}>{}</button>",
        attributes(&[("class", class.as_deref()), ("form", form.as_deref())]),
        escape(&label)
    )))
}

/// The hidden form protection field.
fn csrf_field(state: &State) -> Value {
    let token = state
        .lookup("csrf_token")
        .and_then(|token| token.as_str().map(str::to_string))
        .unwrap_or_default();
    Value::from_safe_string(format!(
        r#"<input type="hidden" name="csrf_token" value="{}">"#,
        escape(&token)
    ))
}

/// A form with one button that posts to `action`, for actions that change
/// something ("Smazat", "↑"). `confirm` asks before sending.
fn action_button(state: &State, label: String, action: String, kwargs: Kwargs) -> Result<Value, Error> {
    let class = optional(&kwargs, "class")?;
    let style = optional(&kwargs, "style")?;
    let confirm = optional(&kwargs, "confirm")?;
    let aria_label = optional(&kwargs, "aria_label")?;
    kwargs.assert_all_used()?;
    Ok(Value::from_safe_string(format!(
        r#"<form class="action-button" method="post"{}>{}<button type="submit"{}>{}</button></form>"#,
        attributes(&[("action", Some(&action)), ("data-turbo-confirm", confirm.as_deref())]),
        csrf_field(state),
        attributes(&[
            ("class", class.as_deref()),
            ("style", style.as_deref()),
            ("aria-label", aria_label.as_deref()),
        ]),
        escape(&label)
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::record::Values;

    fn render(template: &str, f: &FormModel) -> String {
        let mut env = Environment::new();
        register(&mut env);
        env.render_str(template, minijinja::context! { f => f, csrf_token => "TOKEN" })
            .unwrap()
    }

    fn form(errors: &[&str]) -> FormModel {
        let mut values = Values::default();
        values.set("key", Some("smrk".into()));
        values.set("title", Some("Nový <b>".into()));
        values.set("hardness", None);
        values.set("position", None);
        values.set("active", Some("1".into()));
        let mut list = Errors::new();
        for field in errors {
            list.add(*field, format!("{field} is wrong"));
        }
        FormModel::new(values, &list)
    }

    #[test]
    fn fields_are_named_after_the_column_and_escaped() {
        assert_eq!(
            render(r#"{{ text_field(f, "title") }}"#, &form(&[])),
            r#"<input type="text" name="title" id="title" value="Nový &lt;b&gt;">"#
        );
        // Nothing stored: no value attribute at all.
        assert_eq!(
            render(r#"{{ number_field(f, "position") }}"#, &form(&[])),
            r#"<input type="number" name="position" id="position">"#
        );
    }

    #[test]
    fn fields_with_errors_are_marked() {
        let f = form(&["key"]);
        assert_eq!(f.error_count, "1 chyba");
        assert_eq!(
            render(r#"{{ text_field(f, "key") }}"#, &f),
            r#"<input type="text" name="key" id="key" class="field-error" value="smrk">"#
        );
    }

    #[test]
    fn selects_mark_the_current_choice_and_blank_means_none() {
        assert_eq!(
            render(
                r#"{{ select(f, "hardness", [["— nepoužito —", none], ["Tvrdé", "hard"]]) }}"#,
                &form(&[])
            ),
            r#"<select name="hardness" id="hardness"><option value="" selected="selected">— nepoužito —</option><option value="hard">Tvrdé</option></select>"#
        );
    }

    #[test]
    fn check_boxes_and_action_buttons() {
        assert_eq!(
            render(r#"{{ check_box(f, "active") }}"#, &form(&[])),
            r#"<input type="checkbox" value="1" name="active" id="active" checked="checked">"#
        );
        assert_eq!(
            render(
                r#"{{ action_button("Smazat", "/admin/orders/5/delete", class="admin-danger-btn", confirm="Opravdu?") }}"#,
                &form(&[])
            ),
            r#"<form class="action-button" method="post" action="/admin/orders/5/delete" data-turbo-confirm="Opravdu?"><input type="hidden" name="csrf_token" value="TOKEN"><button type="submit" class="admin-danger-btn">Smazat</button></form>"#
        );
    }
}
