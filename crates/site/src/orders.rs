//! Placing an order: the figures (worked out the same way for the summary
//! page and the order itself), saving it with its discount code, and the
//! e-mails to the customer and the sawmill.

use crate::i18n::{t, Locale};
use crate::state::AppState;
use crate::visitor::Visitor;
use domain::cart::{CartLine, Spec};
use domain::checkout::{Payment, Shipping};
use domain::delivery::DeliveryQuote;
use domain::money::Czk;
use domain::vat::{Country, VatBreakdown};
use serde::{Deserialize, Serialize};

/// A cart line as stored on the order (the same JSON as before).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderLine {
    pub catalog_variant_id: i64,
    pub product_key: String,
    pub image: Option<String>,
    pub unit_price_czk: i64,
    pub category: domain::Category,
    pub specs: Vec<Spec>,
    pub quantity: u32,
}

impl From<&CartLine> for OrderLine {
    fn from(l: &CartLine) -> OrderLine {
        OrderLine {
            catalog_variant_id: l.variant_id,
            product_key: l.product_key.clone(),
            image: l.image.clone(),
            unit_price_czk: l.unit_price.get(),
            category: l.category,
            specs: l.specs.clone(),
            quantity: l.quantity,
        }
    }
}

impl OrderLine {
    pub fn as_cart_line(&self) -> CartLine {
        CartLine {
            variant_id: self.catalog_variant_id,
            product_key: self.product_key.clone(),
            image: self.image.clone(),
            category: self.category,
            unit_price: Czk(self.unit_price_czk),
            specs: self.specs.clone(),
            quantity: self.quantity,
        }
    }
}

/// The order as it would be placed now.
#[derive(Clone, Debug)]
pub struct OrderDraft {
    pub lines: Vec<OrderLine>,
    pub subtotal: Czk,
    /// The code and the amount, when it applies.
    pub discount: Option<(String, Czk)>,
    /// Why the cart's code doesn't apply (it stays in the cart).
    pub discount_problem: Option<String>,
    /// Delivery to an address, when the distance could be worked out.
    pub delivery: Option<DeliveryQuote>,
    pub vat: VatBreakdown,
    pub total: Czk,
    pub vat_country: Country,
}

/// The figures for the cart and checkout in `v`.
pub async fn draft(state: &AppState, v: &Visitor) -> OrderDraft {
    let c = &v.checkout;
    let delivery = match c.shipping {
        Some(Shipping::Address) => {
            let to = c.destination();
            match (&to.street, &to.city) {
                (Some(street), Some(city)) => crate::routing::one_way_km(
                    state,
                    street,
                    city,
                    to.zip.as_deref().unwrap_or_default(),
                    Country::or_cz(to.country.as_deref()),
                )
                .await
                .map(|km| DeliveryQuote::new(km, v.shop.czk_per_km)),
                _ => None,
            }
        }
        _ => None,
    };
    let discount = crate::cart_view::applied_discount(v);
    let discount_problem = v.discount.as_ref().and_then(|d| {
        d.unusable_reason(v.cart.subtotal(), time::OffsetDateTime::now_utc().date())
            .map(|reason| t::cart::discount_not_applied(v.locale, &d.code, &crate::forms::cart::discount_reason(v, reason, d.min_order)))
    });
    let vat_country = crate::cart_view::vat_country(v);
    let discount_czk = discount.as_ref().map_or(Czk::ZERO, |(_, a)| *a);
    let delivery_czk = delivery.map_or(Czk::ZERO, DeliveryQuote::price);
    let vat = v.shop.breakdown(&v.cart, vat_country, discount_czk, delivery_czk);
    OrderDraft {
        lines: v.cart.lines.iter().map(OrderLine::from).collect(),
        subtotal: v.cart.subtotal(),
        total: v.cart.subtotal() - discount_czk + delivery_czk,
        discount,
        discount_problem,
        delivery,
        vat,
        vat_country,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlaceError {
    /// Shown to the customer as it is.
    #[error("{0}")]
    Refused(String),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// Saves the order and counts its discount code use (in one transaction:
/// when the code ran out meanwhile, nothing is saved). Returns the order number.
pub async fn place(state: &AppState, v: &Visitor, d: &OrderDraft) -> Result<i64, PlaceError> {
    if let Some(problem) = &d.discount_problem {
        return Err(PlaceError::Refused(problem.clone()));
    }
    let c = &v.checkout;
    let (shipping, payment) = (c.shipping.unwrap_or(Shipping::Pickup).as_str(), c.payment.unwrap_or(Payment::Cash).as_str());
    let items = serde_json::to_string(&d.lines).unwrap_or_else(|_| "[]".into());
    let user_id = v.user.as_ref().map(|u| u.id);
    let locale = v.locale.as_str();
    let (subtotal, total, vat_czk) = (d.subtotal.get(), d.total.get(), d.vat.vat().get());
    let discount_code = d.discount.as_ref().map(|(code, _)| code.clone());
    let discount_czk = d.discount.as_ref().map(|(_, amount)| amount.get());
    let distance = d.delivery.map(DeliveryQuote::round_trip_km);
    let delivery_czk = d.delivery.map(|q| q.price().get());
    let vat_breakdown = d.vat.to_json();
    let vat_country = d.vat_country.as_str();
    let (company, different) = (i64::from(c.company_purchase), i64::from(c.delivery_address_different));
    let newsletter = i64::from(c.newsletter_opt_in);

    let mut tx = state.db.begin().await?;
    let id = sqlx::query_scalar!(
        r#"INSERT INTO orders (user_id, locale, items, subtotal_czk, discount_code, discount_czk, delivery_distance_km, delivery_czk,
                               vat_czk, vat_breakdown, vat_country, total_czk, shipping_method, payment_method,
                               first_name, last_name, email, phone, billing_street, billing_city, billing_zip, billing_country,
                               company_purchase, company_name, company_ico, company_dic,
                               delivery_address_different, delivery_street, delivery_city, delivery_zip, delivery_country, newsletter_opt_in)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id AS "id!""#,
        user_id, locale, items, subtotal, discount_code, discount_czk, distance, delivery_czk,
        vat_czk, vat_breakdown, vat_country, total, shipping, payment,
        c.first_name, c.last_name, c.email, c.phone, c.billing.street, c.billing.city, c.billing.zip, c.billing.country,
        company, c.company_name, c.company_ico, c.company_dic,
        different, c.delivery.street, c.delivery.city, c.delivery.zip, c.delivery.country, newsletter
    )
    .fetch_one(&mut *tx)
    .await?;
    if let Some(code) = &discount_code {
        // Counted only while the limit allows, so two last uses can't both pass.
        let counted = sqlx::query!(
            "UPDATE discount_codes SET times_used = times_used + 1 WHERE code = ? AND (usage_limit IS NULL OR times_used < usage_limit)",
            code
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if counted != 1 {
            tx.rollback().await?;
            return Err(PlaceError::Refused(t::cart::discount_not_applied(v.locale, code, t::cart::discount_errors::used_up(v.locale))));
        }
    }
    tx.commit().await?;
    Ok(id)
}

/// Keeps the customer's details on their account for next time.
pub async fn save_to_account(state: &AppState, user_id: i64, c: &domain::checkout::CheckoutState) -> Result<(), sqlx::Error> {
    let (company, different) = (i64::from(c.company_purchase), i64::from(c.delivery_address_different));
    sqlx::query!(
        "UPDATE users SET first_name = ?, last_name = ?, phone = ?, billing_street = ?, billing_city = ?, billing_zip = ?, billing_country = ?,
                          company_purchase = ?, company_name = ?, company_ico = ?, company_dic = ?,
                          delivery_address_different = ?, delivery_street = ?, delivery_city = ?, delivery_zip = ?, delivery_country = ?,
                          updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
        c.first_name, c.last_name, c.phone, c.billing.street, c.billing.city, c.billing.zip, c.billing.country,
        company, c.company_name, c.company_ico, c.company_dic,
        different, c.delivery.street, c.delivery.city, c.delivery.zip, c.delivery.country, user_id
    )
    .execute(&state.db)
    .await?;
    Ok(())
}

/// "Česká republika" / "Österreich".
pub fn country_name(l: Locale, code: Option<&str>) -> String {
    match (l, code.unwrap_or_default()) {
        (Locale::Cs, "cz") => "Česká republika".into(),
        (Locale::Cs, "at") => "Rakousko".into(),
        (Locale::De, "cz") => "Tschechien".into(),
        (Locale::De, "at") => "Österreich".into(),
        (_, other) => other.into(),
    }
}

fn shipping_name(l: Locale, shipping: Option<Shipping>) -> &'static str {
    if shipping == Some(Shipping::Pickup) { t::checkout::shipping_pickup(l) } else { t::checkout::shipping_address(l) }
}

fn payment_name(l: Locale, payment: Option<Payment>) -> &'static str {
    if payment == Some(Payment::BankTransfer) { t::checkout::payment_bank_transfer(l) } else { t::checkout::payment_cash(l) }
}

pub fn pay_later_text(l: Locale, payment: Option<Payment>) -> &'static str {
    if payment == Some(Payment::BankTransfer) { t::checkout::pay_later_bank_transfer(l) } else { t::checkout::pay_later_cash(l) }
}

/// The confirmation to the customer (in their language) and the
/// notification to the sawmill (in Czech).
pub fn send_emails(v: &Visitor, d: &OrderDraft, id: i64) {
    let c = &v.checkout;
    let shop = &v.shop;
    let line_text = |l: Locale, per_piece: bool| {
        d.lines
            .iter()
            .map(|line| {
                let cart_line = line.as_cart_line();
                let unit = shop.price(l, cart_line.unit_price);
                let total = shop.price(l, cart_line.total());
                if per_piece {
                    format!("- {} | {}x | {unit} / ks | celkem {total}\n", shop.line_description(l, &cart_line), line.quantity)
                } else {
                    format!("- {} | {}x | {unit} | {total}\n", shop.line_description(l, &cart_line), line.quantity)
                }
            })
            .collect::<String>()
    };
    let name = format!("{} {}", c.first_name.as_deref().unwrap_or_default(), c.last_name.as_deref().unwrap_or_default());

    // To the customer.
    let l = v.locale;
    let m = |s: &str| s.to_owned();
    let mut text = format!(
        "{}\n\n{}\n\n{}: #{id}\n\n{}\n{}\n\n{}\n{}: {}\n{}: {}\n",
        t::order_mailer::greeting(l, c.first_name.as_deref().unwrap_or_default()),
        m(t::order_mailer::thank_you(l)),
        m(t::order_mailer::order_number(l)),
        t::checkout::pay_later_title(l).to_uppercase(),
        pay_later_text(l, c.payment),
        line_text(l, false),
        m(t::order_mailer::shipping_method(l)),
        shipping_name(l, c.shipping),
        m(t::order_mailer::payment_method(l)),
        payment_name(l, c.payment),
    );
    if let Some((code, amount)) = &d.discount {
        text.push_str(&format!("{}: −{}\n", t::order_mailer::discount(l, code), shop.price(l, *amount)));
    }
    if let Some(q) = d.delivery {
        text.push_str(&format!("{}: {}\n", t::order_mailer::delivery(l, q.round_trip_km()), shop.price(l, q.price())));
    }
    text.push_str(&format!(
        "\n{}: {}\n\n{}\n\n{}\n{}\n",
        t::order_mailer::total_due(l),
        shop.price(l, d.total),
        t::order_mailer::questions_text(l),
        t::order_mailer::sign_off(l),
        t::order_mailer::signature(l)
    ));
    let subject = match l {
        Locale::De => format!("Bestellbestätigung Nr. {id} – Pila Derflice"),
        Locale::Cs => format!("Potvrzení objednávky č. {id} – Pila Derflice"),
    };
    if let Some(email) = c.email.clone() {
        tokio::spawn(crate::mail::send(crate::mail::Mail { to: email, subject, text, html: None }));
    }

    // To the sawmill, in Czech.
    let Some(to) = crate::mail::notification_address(false) else {
        tracing::warn!("ORDER_NOTIFICATION_EMAIL isn't set, so nobody hears about order #{id}");
        return;
    };
    let cs = Locale::Cs;
    let opt = |s: &Option<String>| s.clone().unwrap_or_default();
    let mut text = format!(
        "NOVÁ OBJEDNÁVKA #{id}\n{}\n\nZÁKAZNÍK\nJméno: {}\nE-mail: {}\nTelefon: {}\nNewsletter: {}\n\nFAKTURAČNÍ ADRESA\n{}\n{}, {}\n{}\n",
        crate::time_fmt::now_cz(),
        name.trim(),
        opt(&c.email),
        opt(&c.phone),
        if c.newsletter_opt_in { "Ano" } else { "Ne" },
        opt(&c.billing.street),
        opt(&c.billing.city),
        opt(&c.billing.zip),
        country_name(cs, c.billing.country.as_deref()),
    );
    if c.company_purchase {
        text.push_str(&format!("FIREMNÍ ÚDAJE\nNázev firmy: {}\nIČO: {}\nDIČ: {}\n", opt(&c.company_name), opt(&c.company_ico), opt(&c.company_dic)));
    }
    if c.delivery_address_different {
        text.push_str(&format!(
            "DORUČOVACÍ ADRESA\n{}\n{}, {}\n{}\n",
            opt(&c.delivery.street),
            opt(&c.delivery.city),
            opt(&c.delivery.zip),
            country_name(cs, c.delivery.country.as_deref())
        ));
    }
    text.push_str(&format!(
        "DOPRAVA A PLATBA\nZpůsob dopravy: {}\nZpůsob platby: {}\n\nOBSAH OBJEDNÁVKY\n{}\n",
        if c.shipping == Some(Shipping::Pickup) { "Osobní odběr na adrese skladu" } else { "Doručení na adresu" },
        if c.payment == Some(Payment::Cash) { "Hotově při vyzvednutí" } else { "Bankovním převodem - proforma" },
        line_text(cs, true)
    ));
    if d.delivery.is_some() || d.discount.is_some() {
        text.push_str(&format!("Zboží: {}\n", shop.price(cs, d.subtotal)));
    }
    if let Some((code, amount)) = &d.discount {
        text.push_str(&format!("{}: −{}\n", t::order_mailer::discount(cs, code), shop.price(cs, *amount)));
    }
    match d.delivery {
        Some(q) => text.push_str(&format!("{}: {}\n", t::order_mailer::delivery(cs, q.round_trip_km()), shop.price(cs, q.price()))),
        None if c.shipping == Some(Shipping::Address) => {
            text.push_str("Vzdálenost se nepodařilo spočítat – cenu dopravy je potřeba kalkulovat individuálně, není v celkové ceně.\n");
        }
        None => {}
    }
    text.push_str(&format!("Celková cena bez DPH: {}\n", shop.price(cs, d.vat.total_excl_vat())));
    for (rate, amount) in &d.vat.by_rate {
        text.push_str(&format!("DPH {rate} % ({}): {}\n", country_name(cs, Some(d.vat_country.as_str())), shop.price(cs, *amount)));
    }
    if d.vat.by_rate.len() > 1 {
        text.push_str(&format!("Celkem DPH: {}\n", shop.price(cs, d.vat.vat())));
    }
    text.push_str(&format!(
        "CELKEM K ÚHRADĚ (včetně DPH): {}\n\nJazyk zákazníka při objednávce: {}\n",
        shop.price(cs, d.total),
        if l == Locale::De { "němčina" } else { "čeština" }
    ));
    tokio::spawn(crate::mail::send(crate::mail::Mail { to, subject: format!("Nová objednávka #{id} – {}", name.trim()), text, html: None }));
}
