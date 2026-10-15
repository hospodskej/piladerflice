//! Rails' form helpers, for the admin templates. Each builds the same HTML
//! as its Rails namesake, so the pages match the Rails admin:
//!
//! - fields of a record's form: `text_field(f, "title")`,
//!   `number_field`, `text_area`, `select(f, "template", options)`,
//!   `check_box`, `file_field`. `f` is the form ([`FormModel`]): the field
//!   names are `catalog_product[title]`, the ids `catalog_product_title`,
//!   and a field with an error is wrapped in `<div class="field_with_errors">`.
//! - loose fields: `text_field_tag(name, value, form=...)`, `hidden_field_tag`
//! - buttons: `submit(label, class=...)`, `button_to(label, url, method=...)`
//!   (a one-button form, e.g. "Smazat"), `token_field()` (the form protection
//!   field every form needs)

use minijinja::value::{Kwargs, Value};
use minijinja::{Environment, Error, ErrorKind, State};
use serde::Serialize;

use super::text::escape;
use crate::admin::record::Values;
use crate::models::validation::Errors;

/// A record's form as the templates get it.
#[derive(Debug, Serialize)]
pub struct FormModel {
    /// "catalog_product"
    pub param: &'static str,
    pub values: Values,
    /// Fields with an error.
    pub error_fields: Vec<String>,
    /// The full messages, for the box above the form.
    pub errors: Vec<String>,
    /// "2 chyby"
    pub error_count: String,
}

impl FormModel {
    pub fn new(param: &'static str, values: Values, errors: &Errors) -> Self {
        Self {
            param,
            values,
            error_fields: errors.fields(),
            errors: errors.messages(),
            error_count: crate::models::validation::error_count(errors.len()),
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
    env.add_function("text_field_tag", text_field_tag);
    env.add_function("hidden_field_tag", hidden_field_tag);
    env.add_function("submit", submit);
    env.add_function("button_to", button_to);
    env.add_function("token_field", token_field);
}

/// The parts of `f` a field needs.
struct Field {
    name: String,
    id: String,
    value: Option<String>,
    has_error: bool,
}

fn field(f: &Value, name: &str) -> Result<Field, Error> {
    let param = f.get_attr("param")?;
    let param = param
        .as_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "the form has no param"))?;
    let value = f.get_attr("values")?.get_item(&Value::from(name))?;
    let value = (!value.is_none() && !value.is_undefined()).then(|| value.to_string());
    let has_error = f
        .get_attr("error_fields")?
        .try_iter()
        .map(|mut fields| fields.any(|field| field.as_str() == Some(name)))
        .unwrap_or(false);
    Ok(Field {
        name: format!("{param}[{name}]"),
        id: format!("{param}_{name}"),
        value,
        has_error,
    })
}

/// Rails' `field_error_proc`: a field with an error gets a red frame.
fn wrap(field: &Field, html: String) -> Value {
    Value::from_safe_string(if field.has_error {
        format!(r#"<div class="field_with_errors">{html}</div>"#)
    } else {
        html
    })
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
    let html = format!(
        "<input{} />",
        attributes(&[
            ("placeholder", placeholder.as_deref()),
            ("type", Some(kind)),
            ("value", field.value.as_deref()),
            ("name", Some(&field.name)),
            ("id", Some(&field.id)),
        ])
    );
    Ok(wrap(&field, html))
}

/// The text starts on a new line, as Rails writes it (browsers drop it).
fn text_area(f: Value, name: String) -> Result<Value, Error> {
    let field = field(&f, &name)?;
    let html = format!(
        "<textarea{}>\n{}</textarea>",
        attributes(&[("name", Some(&field.name)), ("id", Some(&field.id))]),
        escape(field.value.as_deref().unwrap_or_default())
    );
    Ok(wrap(&field, html))
}

/// `options` is a list of [label, value]; a none value is the empty choice.
fn select(f: Value, name: String, options: Value) -> Result<Value, Error> {
    let field = field(&f, &name)?;
    let current = field.value.clone().unwrap_or_default();
    let mut html = format!(
        "<select{}>",
        attributes(&[("name", Some(&field.name)), ("id", Some(&field.id))])
    );
    let mut first = true;
    for option in options.try_iter()? {
        let label = option.get_item(&Value::from(0))?.to_string();
        let value = option.get_item(&Value::from(1))?;
        let value = if value.is_none() {
            String::new()
        } else {
            value.to_string()
        };
        let selected = (value == current && (field.value.is_some() || value.is_empty())).then_some("selected");
        if !first {
            html.push('\n');
        }
        first = false;
        html.push_str(&format!(
            "<option{}>{}</option>",
            attributes(&[("selected", selected), ("value", Some(&value))]),
            escape(&label)
        ));
    }
    html.push_str("</select>");
    Ok(wrap(&field, html))
}

/// A hidden "0" first, so an unticked box still sends a value.
fn check_box(f: Value, name: String) -> Result<Value, Error> {
    let field = field(&f, &name)?;
    let checked = field
        .value
        .as_deref()
        .is_some_and(|value| !matches!(value.to_ascii_lowercase().as_str(), "" | "0" | "f" | "false" | "off"));
    let html = format!(
        r#"<input name="{}" type="hidden" value="0" /><input{} />"#,
        escape(&field.name),
        attributes(&[
            ("type", Some("checkbox")),
            ("value", Some("1")),
            ("checked", checked.then_some("checked")),
            ("name", Some(&field.name)),
            ("id", Some(&field.id)),
        ])
    );
    Ok(wrap(&field, html))
}

fn file_field(f: Value, name: String, kwargs: Kwargs) -> Result<Value, Error> {
    let field = field(&f, &name)?;
    let accept = optional(&kwargs, "accept")?;
    kwargs.assert_all_used()?;
    let html = format!(
        "<input{} />",
        attributes(&[
            ("accept", accept.as_deref()),
            ("type", Some("file")),
            ("name", Some(&field.name)),
            ("id", Some(&field.id)),
        ])
    );
    Ok(wrap(&field, html))
}

/// A field outside a record's form, tied to a form elsewhere on the page
/// with `form=`. No value when there is none.
fn text_field_tag(name: String, value: Option<Value>, kwargs: Kwargs) -> Result<Value, Error> {
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
        "<input{} />",
        attributes(&[
            ("type", Some("text")),
            ("name", Some(&name)),
            ("value", value.as_deref()),
            ("placeholder", placeholder.as_deref()),
            ("form", form.as_deref()),
            ("aria-label", aria_label.as_deref()),
        ])
    )))
}

fn hidden_field_tag(name: String, value: String, kwargs: Kwargs) -> Result<Value, Error> {
    let form = optional(&kwargs, "form")?;
    kwargs.assert_all_used()?;
    Ok(Value::from_safe_string(format!(
        "<input{} />",
        attributes(&[
            ("type", Some("hidden")),
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
        "<input{} />",
        attributes(&[
            ("type", Some("submit")),
            ("name", Some("commit")),
            ("value", Some(&label)),
            ("form", form.as_deref()),
            ("class", class.as_deref()),
            ("data-disable-with", Some(&label)),
        ])
    )))
}

fn csrf_token(state: &State) -> String {
    state
        .lookup("csrf_token")
        .and_then(|token| token.as_str().map(str::to_string))
        .unwrap_or_default()
}

/// The hidden form protection field.
fn token_field(state: &State) -> Value {
    Value::from_safe_string(format!(
        r#"<input type="hidden" name="csrf_token" value="{}" />"#,
        escape(&csrf_token(state))
    ))
}

/// A form with one button, for actions that change something ("Smazat",
/// "↑"). `method` adds Rails' `_method`; `form_confirm` asks before
/// sending (on the form), `confirm` the same on the button.
fn button_to(state: &State, label: String, action: String, kwargs: Kwargs) -> Result<Value, Error> {
    let method = optional(&kwargs, "method")?;
    let class = optional(&kwargs, "class")?;
    let style = optional(&kwargs, "style")?;
    let form_confirm = optional(&kwargs, "form_confirm")?;
    let confirm = optional(&kwargs, "confirm")?;
    let aria_label = optional(&kwargs, "aria_label")?;
    kwargs.assert_all_used()?;
    let method_field = method
        .map(|method| format!(r#"<input type="hidden" name="_method" value="{}" />"#, escape(&method)))
        .unwrap_or_default();
    Ok(Value::from_safe_string(format!(
        r#"<form{}>{method_field}<button{}>{}</button>{}</form>"#,
        attributes(&[
            ("data-turbo-confirm", form_confirm.as_deref()),
            ("class", Some("button_to")),
            ("method", Some("post")),
            ("action", Some(&action)),
        ]),
        attributes(&[
            ("class", class.as_deref()),
            ("style", style.as_deref()),
            ("data-turbo-confirm", confirm.as_deref()),
            ("aria-label", aria_label.as_deref()),
            ("type", Some("submit")),
        ]),
        escape(&label),
        token_field(state)
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
        values.set("active", Some("1".into()));
        let mut list = Errors::default();
        for field in errors {
            list.push(field, format!("{field} is wrong"));
        }
        FormModel::new("catalog_product", values, &list)
    }

    #[test]
    fn fields_are_named_after_the_record_and_escaped() {
        assert_eq!(
            render(r#"{{ text_field(f, "title") }}"#, &form(&[])),
            r#"<input type="text" value="Nový &lt;b&gt;" name="catalog_product[title]" id="catalog_product_title" />"#
        );
        // Nothing stored: no value attribute at all.
        assert_eq!(
            render(r#"{{ number_field(f, "position") }}"#, &form(&[])),
            r#"<input type="number" name="catalog_product[position]" id="catalog_product_position" />"#
        );
    }

    #[test]
    fn fields_with_errors_are_framed() {
        assert_eq!(
            render(r#"{{ text_field(f, "key") }}"#, &form(&["key"])),
            r#"<div class="field_with_errors"><input type="text" value="smrk" name="catalog_product[key]" id="catalog_product_key" /></div>"#
        );
    }

    #[test]
    fn selects_mark_the_current_choice_and_blank_means_none() {
        let html = render(
            r#"{{ select(f, "hardness", [["— nepoužito —", none], ["Tvrdé", "hard"]]) }}"#,
            &form(&[]),
        );
        assert_eq!(
            html,
            "<select name=\"catalog_product[hardness]\" id=\"catalog_product_hardness\"><option selected=\"selected\" value=\"\">— nepoužito —</option>\n<option value=\"hard\">Tvrdé</option></select>"
        );
    }

    #[test]
    fn check_boxes_send_zero_when_unticked() {
        assert_eq!(
            render(r#"{{ check_box(f, "active") }}"#, &form(&[])),
            r#"<input name="catalog_product[active]" type="hidden" value="0" /><input type="checkbox" value="1" checked="checked" name="catalog_product[active]" id="catalog_product_active" />"#
        );
    }

    #[test]
    fn one_button_forms_carry_the_method_and_the_token() {
        assert_eq!(
            render(
                r#"{{ button_to("Smazat", "/admin/orders/5", method="delete", class="admin-danger-btn", form_confirm="Opravdu?") }}"#,
                &form(&[])
            ),
            r#"<form data-turbo-confirm="Opravdu?" class="button_to" method="post" action="/admin/orders/5"><input type="hidden" name="_method" value="delete" /><button class="admin-danger-btn" type="submit">Smazat</button><input type="hidden" name="csrf_token" value="TOKEN" /></form>"#
        );
    }
}
