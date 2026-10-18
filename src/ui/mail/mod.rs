//! The e-mails' HTML: a simple table layout with inline styles, as e-mail
//! programs want it. The data comes ready from `mail/` ([`OrderMail`],
//! [`InquiryMail`]); the plain text versions are built there too.

mod inquiry;
mod order;

pub use inquiry::NewInquiry;
pub use order::{CustomerConfirmation, NewOrder};

use leptos::prelude::*;

pub use crate::mail::inquiries::InquiryMail;
pub use crate::mail::orders::OrderMail;

/// A section heading of the business's e-mails.
const H2: &str = "font-size:15px; margin:0 0 8px 0; border-bottom:1px solid #eee; padding-bottom:6px";
/// A label cell of their tables.
const LABEL: &str = "padding:2px 0; color:#666";
const VALUE: &str = "padding:2px 0";
const TABLE: &str = "font-size:14px; margin-bottom:20px";

#[component]
pub fn MailLayout(children: Children) -> impl IntoView {
    view! {
        <html>
            <head>
                <meta content="text/html; charset=UTF-8" http-equiv="Content-Type"/>
            </head>
            <body style="margin:0; padding:0; background-color:#f4f4f4; font-family: Arial, Helvetica, sans-serif; color:#222">
                <MailTable style="background-color:#f4f4f4; padding:24px 0">
                    <tr>
                        <Centered>
                            <MailTable width="600" style="background-color:#ffffff; border-radius:8px; overflow:hidden">
                                <tr>
                                    <td style="background-color:#1f2937; padding:20px 30px">
                                        <span style="color:#ffffff; font-size:18px; font-weight:bold">"Pila Derflice"</span>
                                    </td>
                                </tr>
                                <tr>
                                    <td style="padding:30px">{children()}</td>
                                </tr>
                            </MailTable>
                        </Centered>
                    </tr>
                </MailTable>
            </body>
        </html>
    }
}

/// A layout table (`role="presentation"`), with the old attributes
/// e-mail programs still go by.
#[component]
fn MailTable(
    #[prop(into)] style: String,
    #[prop(default = "100%")] width: &'static str,
    children: Children,
) -> impl IntoView {
    leptos::html::table()
        .attr("role", "presentation")
        .attr("width", width)
        .attr("cellpadding", "0")
        .attr("cellspacing", "0")
        .attr("style", style)
        .child(children())
}

/// A cell that centers its content (`align`, for e-mail programs).
#[component]
fn Centered(children: Children) -> impl IntoView {
    leptos::html::td().attr("align", "center").child(children())
}

/// A table of label/value rows (the business's e-mails).
fn rows(rows: Vec<(&'static str, String)>) -> impl IntoView {
    let rows = rows
        .into_iter()
        .enumerate()
        .map(|(index, (label, value))| {
            // The first row sets the label column's width.
            let label_style = if index == 0 {
                format!("{LABEL}; width:140px")
            } else {
                LABEL.to_string()
            };
            view! {
                <tr>
                    <td style=label_style>{label}</td>
                    <td style=VALUE>{value}</td>
                </tr>
            }
        })
        .collect_view();
    view! {
        <MailTable style=TABLE>
            {rows}
        </MailTable>
    }
}
