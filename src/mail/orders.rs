//! The two e-mails of a new order: the confirmation for the customer (in
//! their language) and the notification for the business (in Czech). The
//! HTML is `ui::mail`; the plain text is written here.

use std::fmt::Write;

use leptos::prelude::*;

use super::{Email, render_html};
use crate::AppState;
use crate::locale::Locale;
use crate::models::order::{Order, country_name};
use crate::money::Money;
use crate::shop::describe::Describer;
use crate::ui::mail::{CustomerConfirmation, NewOrder};

/// An order as its e-mails show it, in one language and currency.
#[derive(Clone)]
pub struct OrderMail {
    pub id: i64,
    /// "18.10.2026 17:57", Czech time.
    pub created_at: String,
    pub first_name: String,
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub newsletter: bool,
    pub billing: MailAddress,
    /// Name, IČO, DIČ of a company purchase.
    pub company: Option<(String, String, String)>,
    pub delivery_address: Option<MailAddress>,
    pub pickup: bool,
    /// Delivery to an address that couldn't be priced.
    pub by_address: bool,
    pub cash: bool,
    pub bank_transfer: bool,
    /// The round trip in km, and its price, when the delivery was priced.
    pub delivery: Option<(i64, String)>,
    pub lines: Vec<MailLine>,
    pub subtotal: String,
    pub without_vat: String,
    pub vat: String,
    pub total: String,
    pub german_customer: bool,
}

#[derive(Clone)]
pub struct MailAddress {
    pub street: String,
    pub city: String,
    pub zip: String,
    /// "Česká republika"
    pub country: String,
}

#[derive(Clone)]
pub struct MailLine {
    pub description: String,
    pub quantity: i64,
    pub unit_price: String,
    pub line_total: String,
}

impl OrderMail {
    async fn load(state: &AppState, order: &Order, locale: Locale) -> anyhow::Result<Self> {
        let money = state.rates.money(locale).await;
        let describer = Describer::load(&state.db, &state.translations).await?;
        let text = |value: &Option<String>| value.clone().unwrap_or_default();
        let address = |street, city, zip, country: &Option<String>| MailAddress {
            street: text(street),
            city: text(city),
            zip: text(zip),
            country: country_name(country.as_deref(), locale),
        };
        let delivery = |money: &Money| {
            order.delivery_quoted().then(|| {
                (
                    order.delivery_distance_km.unwrap_or_default(),
                    money.display(order.delivery_czk.unwrap_or_default()),
                )
            })
        };
        Ok(Self {
            id: order.id,
            created_at: crate::czech_time::from_utc(order.created_at)
                .format("%d.%m.%Y %H:%M")
                .to_string(),
            first_name: text(&order.first_name),
            full_name: order.full_name(),
            email: text(&order.email),
            phone: text(&order.phone),
            newsletter: order.newsletter_opt_in,
            billing: address(
                &order.billing_street,
                &order.billing_city,
                &order.billing_zip,
                &order.billing_country,
            ),
            company: order.company_purchase.then(|| {
                (
                    text(&order.company_name),
                    text(&order.company_ico),
                    text(&order.company_dic),
                )
            }),
            delivery_address: order.delivery_address_different.then(|| {
                address(
                    &order.delivery_street,
                    &order.delivery_city,
                    &order.delivery_zip,
                    &order.delivery_country,
                )
            }),
            pickup: order.shipping_method.as_deref() == Some("pickup"),
            by_address: order.shipping_method.as_deref() == Some("address") && !order.delivery_quoted(),
            cash: order.payment_method.as_deref() == Some("cash"),
            bank_transfer: order.payment_method.as_deref() == Some("bank_transfer"),
            delivery: delivery(&money),
            lines: order
                .items()
                .iter()
                .map(|item| MailLine {
                    description: describer.describe(item, locale),
                    quantity: item.quantity,
                    unit_price: money.display(item.unit_price_czk),
                    line_total: money.display(item.line_total_czk()),
                })
                .collect(),
            subtotal: money.display(order.subtotal_czk),
            without_vat: money.display(order.total_czk - order.vat_czk),
            vat: money.display(order.vat_czk),
            total: money.display(order.total_czk),
            german_customer: order.locale == "de",
        })
    }

    /// For the business (always Czech).
    pub fn shipping_text(&self) -> &'static str {
        if self.pickup {
            "Osobní odběr na adrese skladu"
        } else {
            "Doručení na adresu"
        }
    }

    pub fn payment_text(&self) -> &'static str {
        if self.cash {
            "Hotově při vyzvednutí"
        } else {
            "Bankovním převodem - proforma"
        }
    }

    pub fn customer_language(&self) -> &'static str {
        if self.german_customer { "němčina" } else { "čeština" }
    }
}

/// Thank-you e-mail to the customer.
pub async fn customer_confirmation(state: &AppState, order: &Order) -> anyhow::Result<Email> {
    let locale = Locale::from_code(&order.locale).unwrap_or(Locale::Cs);
    let mail = OrderMail::load(state, order, locale).await?;
    let subject = match locale {
        Locale::De => format!("Bestellbestätigung Nr. {} – Pila Derflice", order.id),
        Locale::Cs => format!("Potvrzení objednávky č. {} – Pila Derflice", order.id),
    };
    let text = customer_text(state, locale, &mail);
    let html = {
        let mail = mail.clone();
        render_html(state, locale, move || view! { <CustomerConfirmation order=mail/> })
    };
    Ok(Email {
        to: mail.email,
        subject,
        html,
        text,
    })
}

/// Notification of a new order for the business, always in Czech.
pub async fn new_order(state: &AppState, order: &Order) -> anyhow::Result<Email> {
    let mail = OrderMail::load(state, order, Locale::Cs).await?;
    let subject = format!("Nová objednávka #{} – {}", order.id, order.full_name());
    let text = new_order_text(state, &mail);
    let html = render_html(state, Locale::Cs, move || view! { <NewOrder order=mail/> });
    Ok(Email {
        to: state.mail_settings.order_notification_to.clone(),
        subject,
        html,
        text,
    })
}

fn customer_text(state: &AppState, locale: Locale, mail: &OrderMail) -> String {
    let t = |key: &str| state.translations.text(locale, key, &[]);
    let mut text = String::new();
    let _ = writeln!(
        text,
        "{}\n",
        state
            .translations
            .text(locale, "order_mailer.greeting", &[("name", &mail.first_name)])
    );
    let _ = writeln!(text, "{}\n", t("order_mailer.thank_you"));
    let _ = writeln!(text, "{}: #{}\n", t("order_mailer.order_number"), mail.id);
    for line in &mail.lines {
        let _ = writeln!(
            text,
            "- {} | {}x | {} | {}",
            line.description, line.quantity, line.unit_price, line.line_total
        );
    }
    let shipping = t(if mail.pickup {
        "checkout.shipping_pickup"
    } else {
        "checkout.shipping_address"
    });
    let payment = t(if mail.bank_transfer {
        "checkout.payment_bank_transfer"
    } else {
        "checkout.payment_cash"
    });
    let _ = writeln!(text, "\n{}: {shipping}", t("order_mailer.shipping_method"));
    let _ = writeln!(text, "{}: {payment}", t("order_mailer.payment_method"));
    if let Some((km, price)) = &mail.delivery {
        let label = state
            .translations
            .text(locale, "order_mailer.delivery", &[("km", &km.to_string())]);
        let _ = writeln!(text, "{label}: {price}");
    }
    let _ = writeln!(text, "\n{}: {}\n", t("order_mailer.total_due"), mail.total);
    let _ = writeln!(text, "{}\n", t("order_mailer.questions_text"));
    let _ = write!(text, "{}\n{}", t("order_mailer.sign_off"), t("order_mailer.signature"));
    text
}

fn new_order_text(state: &AppState, mail: &OrderMail) -> String {
    let mut text = String::new();
    let _ = writeln!(text, "NOVÁ OBJEDNÁVKA #{}\n{}\n", mail.id, mail.created_at);
    let _ = writeln!(
        text,
        "ZÁKAZNÍK\nJméno: {}\nE-mail: {}\nTelefon: {}",
        mail.full_name, mail.email, mail.phone
    );
    let _ = writeln!(text, "Newsletter: {}\n", if mail.newsletter { "Ano" } else { "Ne" });
    let address = |text: &mut String, address: &MailAddress| {
        let _ = writeln!(
            text,
            "{}\n{}, {}\n{}",
            address.street, address.city, address.zip, address.country
        );
    };
    let _ = writeln!(text, "FAKTURAČNÍ ADRESA");
    address(&mut text, &mail.billing);
    if let Some((name, ico, dic)) = &mail.company {
        let _ = writeln!(text, "FIREMNÍ ÚDAJE\nNázev firmy: {name}\nIČO: {ico}\nDIČ: {dic}");
    }
    if let Some(delivery) = &mail.delivery_address {
        let _ = writeln!(text, "DORUČOVACÍ ADRESA");
        address(&mut text, delivery);
    }
    let _ = writeln!(
        text,
        "DOPRAVA A PLATBA\nZpůsob dopravy: {}\nZpůsob platby: {}\n",
        mail.shipping_text(),
        mail.payment_text()
    );
    let _ = writeln!(text, "OBSAH OBJEDNÁVKY");
    for line in &mail.lines {
        let _ = writeln!(
            text,
            "- {} | {}x | {} / ks | celkem {}",
            line.description, line.quantity, line.unit_price, line.line_total
        );
    }
    let _ = writeln!(text);
    if let Some((km, price)) = &mail.delivery {
        let label = state
            .translations
            .text(Locale::Cs, "order_mailer.delivery", &[("km", &km.to_string())]);
        let _ = writeln!(text, "Zboží: {}\n{label}: {price}", mail.subtotal);
    } else if mail.by_address {
        let _ = writeln!(
            text,
            "Vzdálenost se nepodařilo spočítat – cenu dopravy je potřeba kalkulovat individuálně, není v celkové ceně."
        );
    }
    let _ = writeln!(
        text,
        "Celková cena bez DPH: {}\nCelkem DPH: {}",
        mail.without_vat, mail.vat
    );
    let _ = writeln!(text, "CELKEM K ÚHRADĚ (včetně DPH): {}\n", mail.total);
    let _ = write!(text, "Jazyk zákazníka při objednávce: {}", mail.customer_language());
    text
}
