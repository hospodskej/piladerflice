//! The order e-mails: the customer's confirmation (in their language) and
//! the business's notification (Czech).

use leptos::prelude::*;

use super::{H2, LABEL, MailTable, OrderMail, VALUE, rows};
use crate::ui::{t, t_with};

/// The order lines; `headers`: item, quantity, unit price, total.
fn lines_table(order: &OrderMail, headers: [String; 4]) -> impl IntoView + use<> {
    let [item, quantity, unit_price, total] = headers;
    let head = "padding:6px 0; color:#666; font-size:12px";
    let lines = order
        .lines
        .iter()
        .map(|line| {
            view! {
                <tr style="border-bottom:1px solid #f0f0f0">
                    <td style="padding:8px 0">{line.description.clone()}</td>
                    <td style="padding:8px 0; text-align:center">{line.quantity}</td>
                    <td style="padding:8px 0; text-align:right; white-space:nowrap">{line.unit_price.clone()}</td>
                    <td style="padding:8px 0; text-align:right; white-space:nowrap">{line.line_total.clone()}</td>
                </tr>
            }
        })
        .collect_view();
    view! {
        <MailTable style="font-size:14px; margin-bottom:20px; border-collapse:collapse">
            <tr style="border-bottom:1px solid #ddd">
                <td style=head>{item}</td>
                <td style=format!("{head}; text-align:center")>{quantity}</td>
                <td style=format!("{head}; text-align:right")>{unit_price}</td>
                <td style=format!("{head}; text-align:right")>{total}</td>
            </tr>
            {lines}
        </MailTable>
    }
}

/// "Doprava (50 km tam i zpět, orientačně)"
fn delivery_label(km: i64) -> String {
    t_with("order_mailer.delivery", &[("km", &km.to_string())])
}

#[component]
pub fn CustomerConfirmation(order: OrderMail) -> impl IntoView {
    let headers = [
        t("order_mailer.item"),
        t("order_mailer.quantity"),
        t("order_mailer.unit_price"),
        t("order_mailer.total"),
    ];
    let shipping = t(if order.pickup {
        "checkout.shipping_pickup"
    } else {
        "checkout.shipping_address"
    });
    let payment = t(if order.bank_transfer {
        "checkout.payment_bank_transfer"
    } else {
        "checkout.payment_cash"
    });
    view! {
        <h1 style="font-size:20px; margin:0 0 16px 0">{t_with("order_mailer.greeting", &[("name", &order.first_name)])}</h1>
        <p style="margin:0 0 20px 0">{t("order_mailer.thank_you")}</p>
        <p style="margin:0 0 20px 0; font-size:14px; color:#666">
            {format!("{}: ", t("order_mailer.order_number"))}
            <strong>{format!("#{}", order.id)}</strong>
        </p>
        {lines_table(&order, headers)}
        <MailTable style="font-size:14px; margin-bottom:20px">
            <tr>
                <td style=format!("{LABEL}; width:160px")>{t("order_mailer.shipping_method")}</td>
                <td style=VALUE>{shipping}</td>
            </tr>
            <tr>
                <td style=LABEL>{t("order_mailer.payment_method")}</td>
                <td style=VALUE>{payment}</td>
            </tr>
            {order
                .delivery
                .clone()
                .map(|(km, price)| {
                    view! {
                        <tr>
                            <td style=LABEL>{delivery_label(km)}</td>
                            <td style=VALUE>{price}</td>
                        </tr>
                    }
                })}
            <tr>
                <td style="padding:10px 0 0 0; font-weight:bold; font-size:16px">{t("order_mailer.total_due")}</td>
                <td style="padding:10px 0 0 0; font-weight:bold; font-size:16px">{order.total.clone()}</td>
            </tr>
        </MailTable>
        <p style="margin:24px 0 0 0; color:#444; font-size:14px">{t("order_mailer.questions_text")}</p>
        <p style="margin:20px 0 0 0; font-size:14px">
            {t("order_mailer.sign_off")}
            <br/>
            {t("order_mailer.signature")}
        </p>
    }
}

#[component]
pub fn NewOrder(order: OrderMail) -> impl IntoView {
    let headers = ["Položka", "Množství", "Jedn. cena", "Celkem"].map(str::to_string);
    let lines = lines_table(&order, headers);
    let (shipping, payment, language) = (order.shipping_text(), order.payment_text(), order.customer_language());
    let OrderMail {
        id,
        created_at,
        full_name,
        email,
        phone,
        newsletter,
        billing,
        company,
        delivery_address,
        by_address,
        delivery,
        subtotal,
        without_vat,
        vat,
        total,
        ..
    } = order;
    let address = |street: &str, city: &str, zip: &str, country: &str| {
        rows(vec![
            ("Ulice a č.p.", street.to_string()),
            ("Město", city.to_string()),
            ("PSČ", zip.to_string()),
            ("Stát", country.to_string()),
        ])
    };
    let mailto = format!("mailto:{email}");
    let right = format!("{VALUE}; text-align:right");
    let totals = match (delivery, by_address) {
        (Some((km, price)), _) => Some(
            view! {
                <tr>
                    <td style=LABEL>"Zboží"</td>
                    <td style=right.clone()>{subtotal}</td>
                </tr>
                <tr>
                    <td style=LABEL>{delivery_label(km)}</td>
                    <td style=right.clone()>{price}</td>
                </tr>
            }
            .into_any(),
        ),
        (None, true) => Some(
            view! {
                <tr>
                    <td colspan="2" style="padding:2px 0 8px; color:#a15c00">
                        "Vzdálenost se nepodařilo spočítat – cenu dopravy je potřeba kalkulovat individuálně, není v celkové ceně."
                    </td>
                </tr>
            }
            .into_any(),
        ),
        (None, false) => None,
    };
    view! {
        <h1 style="font-size:20px; margin:0 0 4px 0">{format!("Nová objednávka #{}", id)}</h1>
        <p style="color:#666; margin:0 0 24px 0; font-size:13px">{format!("Přijato {}", created_at)}</p>
        <h2 style=H2>"Zákazník"</h2>
        <MailTable style="font-size:14px; margin-bottom:20px">
            <tr>
                <td style=format!("{LABEL}; width:140px")>"Jméno"</td>
                <td style=VALUE>{full_name}</td>
            </tr>
            <tr>
                <td style=LABEL>"E-mail"</td>
                <td style=VALUE>
                    <a href=mailto>{email}</a>
                </td>
            </tr>
            <tr>
                <td style=LABEL>"Telefon"</td>
                <td style=VALUE>{phone}</td>
            </tr>
            <tr>
                <td style=LABEL>"Newsletter"</td>
                <td style=VALUE>{if newsletter { "Ano" } else { "Ne" }}</td>
            </tr>
        </MailTable>
        <h2 style=H2>"Fakturační adresa"</h2>
        {address(&billing.street, &billing.city, &billing.zip, &billing.country)}
        {company
            .map(|(name, ico, dic)| {
                view! {
                    <h2 style=H2>"Firemní údaje"</h2>
                    {rows(vec![("Název firmy", name), ("IČO", ico), ("DIČ", dic)])}
                }
            })}
        {delivery_address
            .map(|delivery| {
                view! {
                    <h2 style=H2>"Doručovací adresa"</h2>
                    {address(&delivery.street, &delivery.city, &delivery.zip, &delivery.country)}
                }
            })}
        <h2 style=H2>"Doprava a platba"</h2>
        {rows(vec![
            ("Způsob dopravy", shipping.to_string()),
            ("Způsob platby", payment.to_string()),
        ])}
        <h2 style=H2>"Obsah objednávky"</h2>
        {lines}
        <MailTable style="font-size:14px; margin-bottom:10px">
            {totals}
            <tr>
                <td style=LABEL>"Celková cena bez DPH"</td>
                <td style=right.clone()>{without_vat}</td>
            </tr>
            <tr>
                <td style=LABEL>"Celkem DPH"</td>
                <td style=right.clone()>{vat}</td>
            </tr>
            <tr>
                <td style="padding:6px 0; font-weight:bold; font-size:16px">"Celkem k úhradě (včetně DPH)"</td>
                <td style="padding:6px 0; text-align:right; font-weight:bold; font-size:16px">{total}</td>
            </tr>
        </MailTable>
        <p style="color:#999; font-size:12px; margin-top:24px">
            {format!(
                "Objednávka byla odeslána přes web piladerflice.cz. Jazyk zákazníka při objednávce: {}.",
                language,
            )}
        </p>
    }
}
