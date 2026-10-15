//! The shop (Phase 3): cart, form protection, checkout, delivery price,
//! order e-mails and quote requests.

mod common;

use axum::http::{StatusCode, header};
use common::{Browser, TestApp};

const DETAILS: [(&str, &str); 9] = [
    ("_method", "patch"),
    ("first_name", "Jana"),
    ("last_name", "Nováková"),
    ("email", "jana@example.com"),
    ("phone", "602000000"),
    ("billing_street", "Hlavní 1"),
    ("billing_city", "Brno"),
    ("billing_zip", "60200"),
    ("billing_country", "cz"),
];

/// A browser with 2× the smrk 25 cm / 5 PRM container (9 000 Kč) in the cart.
async fn with_cart(app: &TestApp) -> Browser<'_> {
    let mut browser = app.browser();
    browser.get("/eshop/smrk").await;
    let page = browser
        .post("/cart_items", &[("catalog_variant_id", "10"), ("quantity", "2")])
        .await;
    assert_eq!(page.status, StatusCode::SEE_OTHER);
    browser
}

async fn through_details(browser: &mut Browser<'_>, shipping: &str, extra: &[(&str, &str)]) {
    browser.get("/kosik/doprava").await;
    let page = browser
        .post(
            "/kosik/doprava",
            &[
                ("_method", "patch"),
                ("shipping_method", shipping),
                ("payment_method", "cash"),
            ],
        )
        .await;
    assert_eq!(page.location(), "/kosik/udaje");
    browser.get("/kosik/udaje").await;
    let mut fields: Vec<(&str, &str)> = DETAILS.to_vec();
    fields.extend_from_slice(extra);
    let page = browser.post("/kosik/udaje", &fields).await;
    assert_eq!(page.location(), "/kosik/souhrn");
}

#[tokio::test]
async fn adding_to_the_cart_fills_the_header_widget() {
    let app = TestApp::new().await;
    let mut browser = with_cart(&app).await;
    let page = browser.get("/kosik").await;
    page.assert_contains(r#"<span class="cart-item-desc">Smrk, Sypané, 25 cm, 5 PRM</span>"#);
    page.assert_contains(r#"<span class="cart-item-price">18 000 Kč</span>"#);
    page.assert_contains(r#"<input type="number" name="quantity" value="2""#);
    page.assert_contains("<strong>18 000 Kč</strong>");
    // 18 000 incl. VAT = 14 876 + 3 124 VAT
    page.assert_contains("<span>3 124 Kč</span>");
    page.assert_contains(r#"src="/uploads/testkey0000000000000000000001/smrk%20foto.webp""#);

    // The same variant again adds to the line; a non-number quantity is 1.
    browser
        .post("/cart_items", &[("catalog_variant_id", "10"), ("quantity", "x")])
        .await;
    let page = browser.get("/kosik").await;
    page.assert_contains(r#"<input type="number" name="quantity" value="3""#);
}

#[tokio::test]
async fn turbo_gets_the_widget_and_the_added_modal() {
    let app = TestApp::new().await;
    let mut browser = app.browser();
    browser.get("/at/eshop/fosny").await;
    browser.turbo = true;
    let page = browser
        .post("/at/cart_items", &[("catalog_variant_id", "30"), ("quantity", "1")])
        .await;
    assert_eq!(
        page.headers[header::CONTENT_TYPE],
        "text/vnd.turbo-stream.html; charset=utf-8"
    );
    page.assert_contains(r#"<turbo-stream action="replace" target="cart-widget"><template><div id="cart-widget""#);
    page.assert_contains(r#"<turbo-stream action="update" target="cart-added-modal">"#);
    page.assert_contains("Fošny, Qualität AB, 4 m, 200×50 mm");
    page.assert_contains("≈ 356,00 €");
    page.assert_contains(r#"href="/at/kosik""#);
}

#[tokio::test]
async fn quantities_change_and_lines_go() {
    let app = TestApp::new().await;
    let mut browser = with_cart(&app).await;
    browser.get("/kosik").await;
    browser.turbo = true;
    let page = browser
        .post("/cart_items/10", &[("_method", "patch"), ("quantity", "4")])
        .await;
    page.assert_contains(r#"<turbo-stream action="replace" target="cart-page-contents">"#);
    page.assert_contains("36 000 Kč");

    let page = browser.post("/cart_items/10", &[("_method", "delete")]).await;
    page.assert_contains(r#"<p class="cart-empty">"#);
    browser.turbo = false;
    assert_eq!(browser.get("/kosik/doprava").await.location(), "/kosik");
}

#[tokio::test]
async fn hidden_products_cannot_be_added() {
    let app = TestApp::new().await;
    let mut browser = app.browser();
    browser.get("/").await;
    let page = browser.post("/cart_items", &[("catalog_variant_id", "999")]).await;
    assert_eq!(page.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn forms_without_the_right_token_are_refused() {
    let app = TestApp::new().await;
    let mut browser = app.browser();
    browser.get("/eshop/smrk").await;

    let page = browser
        .post_raw("/cart_items", &[("catalog_variant_id", "10".into())])
        .await;
    assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);
    let page = browser
        .post_raw(
            "/cart_items",
            &[
                ("authenticity_token", "guess".into()),
                ("catalog_variant_id", "10".into()),
            ],
        )
        .await;
    assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);

    // A token from another visitor's session doesn't work either.
    let mut other = app.browser();
    other.get("/").await;
    let stolen = other.token().to_string();
    let page = browser
        .post_raw(
            "/cart_items",
            &[("authenticity_token", stolen), ("catalog_variant_id", "10".into())],
        )
        .await;
    assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(browser.get("/kosik").await.body.contains(r#"<p class="cart-empty">"#));
}

#[tokio::test]
async fn forms_posted_from_another_site_are_refused() {
    let app = TestApp::new().await;
    let mut browser = app.browser();
    browser.get("/eshop/smrk").await;
    browser.origin = Some("https://evil.example".into());
    let page = browser.post("/cart_items", &[("catalog_variant_id", "10")]).await;
    assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);
    browser.origin = Some("http://piladerflice.test".into());
    let page = browser.post("/cart_items", &[("catalog_variant_id", "10")]).await;
    assert_eq!(page.status, StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn the_session_cookie_is_encrypted_and_only_set_when_needed() {
    let app = TestApp::new().await;
    let page = app.get("/assets/logo-wm.webp").await;
    assert!(page.headers.get(header::SET_COOKIE).is_none());

    let page = app.get("/").await;
    let cookie = page.headers[header::SET_COOKIE].to_str().unwrap();
    assert!(cookie.starts_with("_pila_session="));
    assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Lax"));
    assert!(!cookie.contains("csrf"), "the session must not be readable");
}

#[tokio::test]
async fn checkout_steps_need_the_steps_before_them() {
    let app = TestApp::new().await;
    let mut browser = app.browser();
    assert_eq!(browser.get("/kosik/doprava").await.location(), "/kosik");
    let mut browser = with_cart(&app).await;
    assert_eq!(browser.get("/kosik/udaje").await.location(), "/kosik/doprava");
    assert_eq!(browser.get("/at/kosik/souhrn").await.location(), "/at/kosik/doprava");
    browser.get("/kosik/doprava").await;
    browser
        .post(
            "/kosik/doprava",
            &[("shipping_method", "pickup"), ("payment_method", "cash")],
        )
        .await;
    assert_eq!(browser.get("/kosik/souhrn").await.location(), "/kosik/udaje");
    assert_eq!(browser.get("/kosik/dekujeme").await.location(), "/kosik");
}

#[tokio::test]
async fn placing_an_order_saves_it_mails_both_sides_and_empties_the_cart() {
    let app = TestApp::new().await;
    let mut browser = with_cart(&app).await;
    through_details(
        &mut browser,
        "pickup",
        &[("newsletter_opt_in", "0"), ("newsletter_opt_in", "1")],
    )
    .await;

    let summary = browser.get("/kosik/souhrn").await;
    summary.assert_contains("Jana Nováková<br>");
    summary.assert_contains("Česká republika");
    summary.assert_lacks("checkout.goods_total");

    let refused = browser.post("/kosik/souhrn", &[]).await;
    assert_eq!(refused.status, StatusCode::UNPROCESSABLE_ENTITY);
    refused.assert_contains("<li>Pro dokončení objednávky je nutné souhlasit s podmínkami.</li>");
    assert!(app.outbox().is_empty());

    let placed = browser.post("/kosik/souhrn", &[("terms_agreement", "1")]).await;
    assert_eq!(placed.location(), "/kosik/dekujeme");

    let order: (i64, String, bool, String, i64, i64, String) = sqlx::query_as(
        "SELECT id, first_name, newsletter_opt_in, shipping_method, total_czk, vat_czk, items_snapshot FROM orders",
    )
    .fetch_one(&app.state.db)
    .await
    .unwrap();
    assert_eq!(
        (order.1.as_str(), order.2, order.3.as_str(), order.4, order.5),
        ("Jana", true, "pickup", 18000, 3124)
    );
    assert!(order.6.contains(r#""catalog_variant_id":"10""#) && order.6.contains(r#""quantity":2"#));

    let thanks = browser.get("/kosik/dekujeme").await;
    thanks.assert_contains(&format!("transaction_id: \"{}\"", order.0));
    thanks.assert_contains("value: 18000.0");
    thanks.assert_contains("mascot/thumbs-up");
    assert!(browser.get("/kosik").await.body.contains(r#"<p class="cart-empty">"#));

    let mails = app.outbox();
    assert_eq!(mails.len(), 2);
    for mail in &mails {
        for body in [&mail.html, &mail.text] {
            assert!(!body.contains("none") && !body.contains("[missing:"), "{body}");
        }
    }
    assert_eq!(mails[0].to, "obchod@piladerflice.test");
    assert_eq!(
        mails[0].subject,
        format!("Nová objednávka #{} – Jana Nováková", order.0)
    );
    assert!(
        mails[0]
            .text
            .contains("- Smrk, Sypané, 25 cm, 5 PRM | 2x | 9 000 Kč / ks | celkem 18 000 Kč")
    );
    assert!(mails[0].text.contains("Newsletter: Ano"));
    assert!(mails[0].html.contains("Osobní odběr na adrese skladu"));
    assert_eq!(mails[1].to, "jana@example.com");
    assert_eq!(
        mails[1].subject,
        format!("Potvrzení objednávky č. {} – Pila Derflice", order.0)
    );
    assert!(mails[1].text.contains("18 000 Kč"));
}

#[tokio::test]
async fn missing_company_or_delivery_details_stop_the_order() {
    let app = TestApp::new().await;
    let mut browser = with_cart(&app).await;
    through_details(
        &mut browser,
        "pickup",
        &[
            ("company_purchase", "1"),
            ("delivery_address_different", "1"),
            ("delivery_city", "Retz"),
        ],
    )
    .await;
    browser.get("/kosik/souhrn").await;
    let page = browser.post("/kosik/souhrn", &[("terms_agreement", "1")]).await;
    assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);
    page.assert_contains("<li>Při nákupu na firmu je potřeba vyplnit název firmy</li>");
    page.assert_contains("<li>Při odlišné doručovací adrese je potřeba vyplnit ulici a č.p.</li>");
    page.assert_lacks("vyplnit město");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orders")
        .fetch_one(&app.state.db)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn delivery_is_priced_by_distance_and_added_to_the_total() {
    let app = TestApp::with_distance(50.0).await;
    let mut browser = with_cart(&app).await;
    through_details(&mut browser, "address", &[]).await;

    let summary = browser.get("/kosik/souhrn").await;
    summary.assert_contains("<strong>4 000 Kč</strong> (orientační cena)");
    summary.assert_contains("<span>Doprava (100 km, orientačně)</span>");
    summary.assert_contains("<strong>22 000 Kč</strong>");

    browser.post("/kosik/souhrn", &[("terms_agreement", "1")]).await;
    let order: (Option<i64>, Option<i64>, i64, i64) =
        sqlx::query_as("SELECT delivery_distance_km, delivery_czk, subtotal_czk, total_czk FROM orders")
            .fetch_one(&app.state.db)
            .await
            .unwrap();
    assert_eq!(order, (Some(100), Some(4000), 18000, 22000));
    let mails = app.outbox();
    assert!(
        mails[1]
            .text
            .contains("Doprava (100 km tam i zpět, orientačně): 4 000 Kč")
    );
}

#[tokio::test]
async fn without_a_distance_delivery_is_priced_individually() {
    let app = TestApp::new().await;
    let mut browser = with_cart(&app).await;
    through_details(&mut browser, "address", &[]).await;
    let summary = browser.get("/kosik/souhrn").await;
    summary.assert_contains("Ceny dopravy jsou kalkulovány individuálně");
    browser.post("/kosik/souhrn", &[("terms_agreement", "1")]).await;
    assert!(app.outbox()[0].text.contains("Vzdálenost se nepodařilo spočítat"));
}

#[tokio::test]
async fn austrian_customers_check_out_in_german_and_euros() {
    let app = TestApp::new().await;
    let mut browser = app.browser();
    browser.get("/at/eshop/smrk").await;
    browser.post("/at/cart_items", &[("catalog_variant_id", "10")]).await;
    browser.get("/at/kosik/doprava").await;
    let page = browser
        .post(
            "/at/kosik/doprava",
            &[("shipping_method", "pickup"), ("payment_method", "cash")],
        )
        .await;
    assert_eq!(page.location(), "/at/kosik/udaje");
    let details = browser.get("/at/kosik/udaje").await;
    details.assert_contains(r#"href="/at/prihlaseni?return_to=%2Fat%2Fkosik%2Fudaje""#);
    browser.post("/at/kosik/udaje", &DETAILS).await;
    let summary = browser.get("/at/kosik/souhrn").await;
    summary.assert_contains("<strong>≈ 360,00 €</strong>");
    let placed = browser.post("/at/kosik/souhrn", &[("terms_agreement", "1")]).await;
    assert_eq!(placed.location(), "/at/kosik/dekujeme");
    let mails = app.outbox();
    assert!(mails[1].subject.starts_with("Bestellbestätigung Nr. "));
    assert!(
        mails[1]
            .text
            .contains("- Fichte, Lose, 25 cm, 5 Rm | 1x | ≈ 360,00 € | ≈ 360,00 €")
    );
    assert!(mails[0].text.contains("Jazyk zákazníka při objednávce: němčina"));
}

#[tokio::test]
async fn a_quote_request_is_saved_mailed_and_confirmed_once() {
    let app = TestApp::new().await;
    let mut browser = app.browser();
    browser.get("/kontakt").await;
    let page = browser
        .post(
            "/inquiries",
            &[
                ("kalkulace_category", "stavebni"),
                ("jmeno", "Petr"),
                ("prijmeni", "Malý"),
                ("email", "petr@example.com"),
                ("telefon", "777111222"),
                ("ulice", "Polní"),
                ("cislo_popisne", "12"),
                ("mesto", "Znojmo"),
                ("psc", "66902"),
                ("polozka[]", "Trámy"),
                ("delka[]", "6 m"),
                ("mnozstvi[]", "4"),
                ("poznamky", "Prosím <b>rychle</b>"),
            ],
        )
        .await;
    assert_eq!(page.location(), "/kontakt#kalkulace");

    let saved: (String, String, String) = sqlx::query_as("SELECT category, items_snapshot, notes FROM inquiries")
        .fetch_one(&app.state.db)
        .await
        .unwrap();
    assert_eq!(saved.0, "stavebni");
    assert_eq!(
        saved.1,
        r#"[{"polozka":"Trámy","delka":"6 m","vyska":null,"sirka":null,"mnozstvi":"4"}]"#
    );

    let mail = &app.outbox()[0];
    assert_eq!(mail.to, "poptavky@piladerflice.test");
    assert!(mail.subject.starts_with("Nová poptávka #") && mail.subject.ends_with("– Petr Malý"));
    assert!(mail.text.contains("Adresa: Polní 12, 66902 Znojmo"));
    assert!(mail.text.contains("- Trámy | 6 m |  |  | 4"));
    assert!(
        mail.html.contains("Prosím &lt;b&gt;rychle&lt;/b&gt;"),
        "typed text is escaped in the HTML mail"
    );

    let page = browser.get("/kontakt").await;
    page.assert_contains(r#"<div class="page-flash page-flash-notice">Děkujeme, poptávku jsme přijali."#);
    browser.get("/kontakt").await.assert_lacks("page-flash");
}

#[tokio::test]
async fn an_incomplete_quote_request_is_not_saved() {
    let app = TestApp::new().await;
    let mut browser = app.browser();
    browser.get("/at/kontakt").await;
    let page = browser.post("/at/inquiries", &[("jmeno", "Petr")]).await;
    assert_eq!(page.location(), "/at/kontakt#kalkulace");
    browser
        .get("/at/kontakt")
        .await
        .assert_contains(r#"<div class="page-flash page-flash-alert">Die Anfrage konnte"#);
    assert!(app.outbox().is_empty());
}
