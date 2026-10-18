//! The business's notification of a new quote request.

use leptos::prelude::*;

use super::{H2, InquiryMail, LABEL, MailTable, VALUE};

#[component]
pub fn NewInquiry(inquiry: InquiryMail) -> impl IntoView {
    let head = "padding:6px 0; color:#666; font-size:12px";
    let notes = (!inquiry.notes.trim().is_empty()).then(|| inquiry.notes.clone());
    view! {
        <h1 style="font-size:20px; margin:0 0 4px 0">{format!("Nová poptávka #{}", inquiry.id)}</h1>
        <p style="color:#666; margin:0 0 24px 0; font-size:13px">
            {format!("Přijato {} (jazyk formuláře: {})", inquiry.created_at, inquiry.language())}
        </p>
        <h2 style=H2>"Kontakt"</h2>
        <MailTable style="font-size:14px; margin-bottom:20px">
            <tr>
                <td style=format!("{LABEL}; width:140px")>"Jméno"</td>
                <td style=VALUE>{inquiry.full_name.clone()}</td>
            </tr>
            <tr>
                <td style=LABEL>"E-mail"</td>
                <td style=VALUE>
                    <a href=format!("mailto:{}", inquiry.email)>{inquiry.email.clone()}</a>
                </td>
            </tr>
            <tr>
                <td style=LABEL>"Telefon"</td>
                <td style=VALUE>{inquiry.phone.clone()}</td>
            </tr>
            {(!inquiry.address.is_empty())
                .then(|| {
                    view! {
                        <tr>
                            <td style=LABEL>"Adresa"</td>
                            <td style=VALUE>{inquiry.address.clone()}</td>
                        </tr>
                    }
                })}
        </MailTable>
        <h2 style=H2>{format!("Požadované položky ({})", inquiry.category)}</h2>
        <MailTable style="font-size:14px; margin-bottom:20px; border-collapse:collapse">
            <tr style="border-bottom:1px solid #ddd">
                {inquiry.headers.iter().map(|header| view! { <td style=head>{*header}</td> }).collect_view()}
            </tr>
            {inquiry
                .rows
                .iter()
                .map(|row| {
                    view! {
                        <tr style="border-bottom:1px solid #f0f0f0">
                            {row.iter().map(|value| view! { <td style="padding:8px 0">{value.clone()}</td> }).collect_view()}
                        </tr>
                    }
                })
                .collect_view()}
        </MailTable>
        {notes
            .map(|notes| {
                view! {
                    <h2 style=H2>"Poznámka"</h2>
                    <p style="font-size:14px; white-space:pre-wrap">{notes}</p>
                }
            })}
        <p style="color:#999; font-size:12px; margin-top:24px">"Poptávka byla odeslána přes formulář na piladerflice.cz/kontakt."</p>
    }
}
