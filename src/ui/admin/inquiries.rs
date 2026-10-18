//! /admin/inquiries: the quote requests from the contact page.

use leptos::prelude::*;

pub struct InquiryRow {
    pub id: i64,
    pub full_name: String,
    pub email: String,
    pub phone: String,
    /// "Palivové dřevo"
    pub category: String,
    pub item_count: usize,
    pub date: String,
}

#[component]
pub fn InquiriesPage(inquiries: Vec<InquiryRow>) -> impl IntoView {
    let list = if inquiries.is_empty() {
        view! { <div class="admin-empty-state">"Zatím žádné poptávky."</div> }.into_any()
    } else {
        view! {
            <table class="admin-table">
                <thead>
                    <tr>
                        <th>"Č."</th>
                        <th>"Jméno"</th>
                        <th>"E-mail"</th>
                        <th>"Telefon"</th>
                        <th>"Kategorie"</th>
                        <th>"Položek"</th>
                        <th>"Datum"</th>
                    </tr>
                </thead>
                <tbody>
                    {inquiries
                        .into_iter()
                        .map(|inquiry| {
                            view! {
                                <tr>
                                    <td>
                                        <a href=format!("/admin/inquiries/{}", inquiry.id)>{format!("#{}", inquiry.id)}</a>
                                    </td>
                                    <td>{inquiry.full_name}</td>
                                    <td>{inquiry.email}</td>
                                    <td>{inquiry.phone}</td>
                                    <td>{inquiry.category}</td>
                                    <td>{inquiry.item_count}</td>
                                    <td>{inquiry.date}</td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        }
        .into_any()
    };
    view! {
        <h1 class="admin-page-title">"Poptávky"</h1>
        {list}
    }
}

pub struct InquiryDetail {
    pub id: i64,
    pub date: String,
    pub german: bool,
    pub full_name: String,
    pub email: String,
    pub phone: String,
    /// Empty without an address.
    pub full_address: String,
    pub category: String,
    pub notes: String,
    /// The items' columns, and each item's values in that order.
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[component]
pub fn InquiryPage(inquiry: InquiryDetail) -> impl IntoView {
    let notes = (!inquiry.notes.trim().is_empty()).then_some(inquiry.notes);
    let address = (!inquiry.full_address.is_empty()).then_some(inquiry.full_address);
    let email = inquiry.email;
    let mailto = format!("mailto:{email}");
    view! {
        <a class="admin-back-link" href="/admin/inquiries">"← Zpět na poptávky"</a>
        <h1 class="admin-page-title">{format!("Poptávka #{}", inquiry.id)}</h1>
        <p class="admin-page-subtitle">
            {format!(
                "{} · jazyk formuláře: {}",
                inquiry.date,
                if inquiry.german { "němčina" } else { "čeština" },
            )}
        </p>
        <div class="admin-detail-grid">
            <div class="admin-box">
                <h2>"Kontakt"</h2>
                <p>
                    {inquiry.full_name}
                    <br/>
                    <a href=mailto>{email}</a>
                    <br/>
                    {inquiry.phone}
                </p>
                {address.map(|address| view! { <p>{address}</p> })}
            </div>
        </div>
        <div class="admin-box">
            <h2>{format!("Požadované položky ({})", inquiry.category)}</h2>
            <table class="admin-table">
                <thead>
                    <tr>{inquiry.headers.into_iter().map(|header| view! { <th>{header}</th> }).collect_view()}</tr>
                </thead>
                <tbody>
                    {inquiry
                        .rows
                        .into_iter()
                        .map(|row| view! { <tr>{row.into_iter().map(|value| view! { <td>{value}</td> }).collect_view()}</tr> })
                        .collect_view()}
                </tbody>
            </table>
            {notes
                .map(|notes| {
                    view! {
                        <h2 style="margin-top: 20px">"Poznámka"</h2>
                        <p style="white-space: pre-wrap">{notes}</p>
                    }
                })}
        </div>
    }
}
