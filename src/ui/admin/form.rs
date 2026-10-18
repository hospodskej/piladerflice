//! The fields of the admin's forms. A record's form ([`Form`]) names each
//! field after its column and fills in the record's value; a field with an
//! error gets the class `field-error`, and [`ErrorsBox`] lists the
//! messages above the form.
//!
//! Some lists edit each row in place: the row's fields are tied to a form
//! elsewhere on the page with `form=` ([`text_input`], [`hidden_input`]).

use std::sync::Arc;

use leptos::prelude::*;

use crate::admin::record::Values;
use crate::forms::Errors;
use crate::islands::admin::ConfirmForm;
use crate::ui::CsrfField;
use crate::views::text::error_count;

/// A record's form: its values as typed, and what is wrong with them.
#[derive(Clone)]
pub struct Form(Arc<Inner>);

struct Inner {
    values: Values,
    errors: Errors,
}

impl Form {
    pub fn new(values: Values, errors: &Errors) -> Self {
        Self(Arc::new(Inner {
            values,
            errors: errors.clone(),
        }))
    }

    pub fn value(&self, name: &str) -> Option<String> {
        self.0.values.get(name).map(str::to_string)
    }

    fn has_error(&self, name: &str) -> bool {
        self.0.errors.has(name)
    }

    /// The class of a field: red with an error.
    fn class(&self, name: &str) -> Option<&'static str> {
        self.has_error(name).then_some("field-error")
    }
}

pub fn text_field(f: &Form, name: &'static str) -> impl IntoView + use<> {
    input(f, name, "text", None)
}

pub fn text_field_with(f: &Form, name: &'static str, placeholder: &'static str) -> impl IntoView + use<> {
    input(f, name, "text", Some(placeholder))
}

pub fn number_field(f: &Form, name: &'static str) -> impl IntoView + use<> {
    input(f, name, "number", None)
}

fn input(f: &Form, name: &'static str, kind: &'static str, placeholder: Option<&'static str>) -> impl IntoView + use<> {
    view! { <input type=kind name=name id=name value=f.value(name) placeholder=placeholder/> }
        .attr("class", f.class(name))
}

pub fn text_area(f: &Form, name: &'static str) -> impl IntoView + use<> {
    view! { <textarea name=name id=name>{f.value(name).unwrap_or_default()}</textarea> }.attr("class", f.class(name))
}

/// `options` are (label, value); a None value is the empty choice.
pub fn select(f: &Form, name: &'static str, options: &[(&'static str, Option<&'static str>)]) -> impl IntoView + use<> {
    let current = f.value(name).unwrap_or_default();
    let options = options
        .iter()
        .map(|(label, value)| {
            let value = value.unwrap_or_default();
            view! { <option value=value selected=value == current>{*label}</option> }
        })
        .collect_view();
    view! { <select name=name id=name>{options}</select> }.attr("class", f.class(name))
}

/// An unticked box sends nothing; the record reads that as false.
pub fn check_box(f: &Form, name: &'static str) -> impl IntoView + use<> {
    let checked = f.value(name).is_some_and(|value| sapwood_forms::de::is_true(&value));
    view! { <input type="checkbox" value="1" name=name id=name checked=checked/> }.attr("class", f.class(name))
}

pub fn file_field(f: &Form, name: &'static str, accept: &'static str) -> impl IntoView + use<> {
    view! { <input type="file" name=name id=name accept=accept/> }.attr("class", f.class(name))
}

/// The box above a form that didn't save.
#[component]
pub fn ErrorsBox(f: Form) -> impl IntoView {
    let messages = f.0.errors.messages();
    (!messages.is_empty()).then(|| {
        view! {
            <div class="admin-flash admin-flash-alert">
                <strong>{format!("{} brání uložení:", error_count(messages.len()))}</strong>
                <ul>{messages.into_iter().map(|message| view! { <li>{message.to_string()}</li> }).collect_view()}</ul>
            </div>
        }
    })
}

/// A field of a row edited in place, tied to the row's form.
pub fn text_input(
    name: String,
    value: Option<String>,
    form: String,
    placeholder: Option<String>,
    aria_label: Option<String>,
) -> impl IntoView {
    view! { <input type="text" name=name value=value placeholder=placeholder form=form aria-label=aria_label/> }
}

pub fn hidden_input(name: &'static str, value: String, form: String) -> impl IntoView {
    view! { <input type="hidden" name=name value=value form=form/> }
}

/// A submit button; `form`: of a form elsewhere on the page.
#[component]
pub fn Submit(label: &'static str, class: &'static str, #[prop(optional)] form: Option<String>) -> impl IntoView {
    view! { <button type="submit" class=class form=form>{label}</button> }
}

/// A form with one button that posts to `action`, for actions that change
/// something ("Smazat", "↑"). `confirm` asks first.
#[component]
pub fn ActionButton(
    label: &'static str,
    action: String,
    class: &'static str,
    #[prop(optional)] style: Option<&'static str>,
    #[prop(optional)] confirm: Option<&'static str>,
    #[prop(optional)] aria_label: Option<&'static str>,
) -> impl IntoView {
    let button = move || {
        view! {
            <CsrfField/>
            <button type="submit" class=class style=style aria-label=aria_label>{label}</button>
        }
    };
    match confirm {
        Some(question) => view! {
            <ConfirmForm action=action question=question.to_string()>{button()}</ConfirmForm>
        }
        .into_any(),
        None => view! { <form class="action-button" method="post" action=action>{button()}</form> }.into_any(),
    }
}
