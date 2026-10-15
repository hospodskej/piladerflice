//! Customer accounts (Phase 4): registering, signing in and out, the
//! profile, the picture, reordering, and checkout filling itself from the
//! account. The fixture accounts are in `fixtures/accounts.sql`.

mod common;

use axum::http::{StatusCode, header};
use common::{Browser, TestApp};

const CUSTOMER: &str = "zakaznik@example.com";

/// A small picture to upload, as PNG.
fn png() -> Vec<u8> {
    let picture = image::RgbImage::from_fn(40, 30, |x, y| image::Rgb([(x * 6) as u8, (y * 8) as u8, 120]));
    let mut bytes = std::io::Cursor::new(Vec::new());
    picture.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
    bytes.into_inner()
}

fn files_in(folder: &std::path::Path) -> usize {
    std::fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .map(|path| if path.is_dir() { files_in(&path) } else { 1 })
        .sum()
}

async fn signed_in<'a>(app: &'a TestApp, email: &str) -> Browser<'a> {
    let mut browser = app.browser();
    let page = browser.sign_in(email).await;
    assert_eq!(page.location(), "/muj-ucet");
    browser
}

#[tokio::test]
async fn the_account_needs_signing_in_and_comes_back_after() {
    let app = TestApp::with_accounts().await;
    let mut browser = app.browser();
    let page = browser.get("/muj-ucet/upravit").await;
    assert_eq!((page.status, page.location()), (StatusCode::SEE_OTHER, "/prihlaseni"));
    let page = browser.get("/prihlaseni").await;
    page.assert_contains("Pro zobrazení účtu se musíte přihlásit.");

    let page = browser
        .post(
            "/prihlaseni",
            &[("email_address", CUSTOMER), ("password", "heslo12345")],
        )
        .await;
    assert_eq!(page.location(), "/muj-ucet/upravit");
    let page = browser.get("/muj-ucet").await;
    assert_eq!(page.status, StatusCode::OK);
    page.assert_contains(CUSTOMER);

    // Austrian visitors are sent to the German sign-in page.
    let page = app.browser().get("/at/muj-ucet").await;
    assert_eq!(page.location(), "/at/prihlaseni");
}

#[tokio::test]
async fn a_wrong_password_or_unknown_email_gets_the_same_answer() {
    let app = TestApp::with_accounts().await;
    let mut browser = app.browser();
    browser.get("/prihlaseni").await;
    for (email, password) in [(CUSTOMER, "spatne-heslo"), ("nikdo@example.com", "heslo12345")] {
        let page = browser
            .post("/prihlaseni", &[("email_address", email), ("password", password)])
            .await;
        assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);
        page.assert_contains("Nesprávný e-mail nebo heslo.");
    }
    assert_eq!(browser.get("/muj-ucet").await.status, StatusCode::SEE_OTHER);

    // E-mails are compared without case and spaces.
    let page = browser
        .post(
            "/prihlaseni",
            &[("email_address", "  ZAKAZNIK@example.COM "), ("password", "heslo12345")],
        )
        .await;
    assert_eq!(page.location(), "/muj-ucet");
}

#[tokio::test]
async fn the_german_site_answers_in_german() {
    let app = TestApp::with_accounts().await;
    let mut browser = app.browser();
    browser.get("/at/prihlaseni").await;
    let page = browser
        .post("/at/prihlaseni", &[("email_address", CUSTOMER), ("password", "falsch")])
        .await;
    page.assert_contains("E-Mail oder Passwort ist falsch.");
    browser.get("/at/registrace").await;
    let page = browser
        .post("/at/registrace", &[("email_address", ""), ("password", "x")])
        .await;
    page.assert_contains("Das Konto konnte nicht erstellt werden:");
    let page = browser.get("/at/muj-ucet").await;
    assert_eq!(page.location(), "/at/prihlaseni");
    browser
        .get("/at/prihlaseni")
        .await
        .assert_contains("Bitte melden Sie sich an, um Ihr Konto zu sehen.");
}

#[tokio::test]
async fn signing_in_returns_only_to_pages_of_this_site() {
    let app = TestApp::with_accounts().await;
    let mut browser = app.browser();
    browser.get("/prihlaseni?return_to=//evil.example").await;
    let page = browser
        .post(
            "/prihlaseni",
            &[("email_address", CUSTOMER), ("password", "heslo12345")],
        )
        .await;
    assert_eq!(page.location(), "/muj-ucet");

    let mut browser = app.browser();
    browser.get("/prihlaseni?return_to=%2Fkosik%2Fudaje").await;
    let page = browser
        .post(
            "/prihlaseni",
            &[("email_address", CUSTOMER), ("password", "heslo12345")],
        )
        .await;
    assert_eq!(page.location(), "/kosik/udaje");
}

#[tokio::test]
async fn signing_in_survives_the_browser_closing_until_signing_out() {
    let app = TestApp::with_accounts().await;
    let mut browser = signed_in(&app, CUSTOMER).await;
    assert!(browser.cookie_header().contains("_pila_login="));

    // The session cookie goes with the browser; the login cookie stays.
    browser.drop_cookie("_pila_session");
    let page = browser.get("/muj-ucet").await;
    assert_eq!(page.status, StatusCode::OK);
    page.assert_contains(r#"href="/muj-ucet""#);

    let page = browser.post("/odhlaseni", &[]).await;
    assert_eq!(page.location(), "/");
    assert!(!browser.cookie_header().contains("_pila_login="));
    browser.get("/").await.assert_contains("Byli jste odhlášeni.");
    assert_eq!(browser.get("/muj-ucet").await.status, StatusCode::SEE_OTHER);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions")
        .fetch_one(&app.state.db)
        .await
        .unwrap();
    assert_eq!(count, 0, "signing out ends the session for good");
}

#[tokio::test]
async fn a_stolen_login_cookie_is_useless_once_signed_out() {
    let app = TestApp::with_accounts().await;
    let mut browser = signed_in(&app, CUSTOMER).await;
    let copied = browser.cookie_header().to_string();
    browser.post("/odhlaseni", &[]).await;

    let mut thief = app.browser();
    thief.set_cookie_header(&copied);
    assert_eq!(thief.get("/muj-ucet").await.status, StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn registering_checks_the_form() {
    let app = TestApp::with_accounts().await;
    let mut browser = app.browser();
    browser.get("/registrace").await;
    let page = browser
        .post(
            "/registrace",
            &[
                ("email_address", CUSTOMER),
                ("password", "kratke"),
                ("password_confirmation", "jine"),
            ],
        )
        .await;
    assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);
    page.assert_contains("3 chyby");
    page.assert_contains("Heslo znovu");
    page.assert_contains("je již zaregistrován");
    page.assert_contains("je příliš krátké");
    page.assert_contains(&format!(r#"value="{CUSTOMER}""#));

    let page = browser
        .post(
            "/registrace",
            &[
                ("email_address", " Nova@Example.com"),
                ("password", "heslo12345"),
                ("password_confirmation", "heslo12345"),
            ],
        )
        .await;
    assert_eq!(page.location(), "/muj-ucet");
    let page = browser.get("/muj-ucet").await;
    page.assert_contains("Účet byl vytvořen.");
    page.assert_contains("nova@example.com");

    let (role, digest): (String, String) =
        sqlx::query_as("SELECT role, password_digest FROM users WHERE email_address = 'nova@example.com'")
            .fetch_one(&app.state.db)
            .await
            .unwrap();
    assert_eq!(role, "customer");
    assert!(digest.starts_with("$2"), "a bcrypt hash");
}

#[tokio::test]
async fn nobody_can_make_themselves_an_admin() {
    let app = TestApp::with_accounts().await;
    let mut browser = app.browser();
    browser.get("/registrace").await;
    browser
        .post(
            "/registrace",
            &[
                ("email_address", "hacker@example.com"),
                ("password", "heslo12345"),
                ("password_confirmation", "heslo12345"),
                ("role", "admin"),
            ],
        )
        .await;
    browser.get("/muj-ucet/upravit").await;
    browser.post("/muj-ucet/upravit", &[("role", "admin")]).await;
    let role: String = sqlx::query_scalar("SELECT role FROM users WHERE email_address = 'hacker@example.com'")
        .fetch_one(&app.state.db)
        .await
        .unwrap();
    assert_eq!(role, "customer");
}

#[tokio::test]
async fn the_profile_is_saved_with_the_phone_put_together() {
    let app = TestApp::with_accounts().await;
    let mut browser = signed_in(&app, CUSTOMER).await;
    let page = browser.get("/muj-ucet/upravit").await;
    page.assert_contains(r#"value="Karel""#);
    page.assert_contains(r#"value="660 1234""#);

    let page = browser
        .post(
            "/muj-ucet/upravit",
            &[
                ("first_name", "Jana"),
                ("phone_prefix", "+420"),
                ("phone_number", " 602 111 222 "),
                ("company_purchase", "1"),
                ("company_name", "Pila s.r.o."),
            ],
        )
        .await;
    assert_eq!(page.location(), "/muj-ucet");
    browser
        .get("/muj-ucet")
        .await
        .assert_contains("Vaše údaje byly uloženy.");

    let saved: (String, String, String, bool, String) =
        sqlx::query_as("SELECT first_name, last_name, phone, company_purchase, company_name FROM users WHERE id = 1")
            .fetch_one(&app.state.db)
            .await
            .unwrap();
    assert_eq!(
        saved,
        (
            "Jana".into(),
            "Dvořák".into(),
            "+420 602 111 222".into(),
            true,
            "Pila s.r.o.".into()
        )
    );
}

/// Regression: empty profile fields once came out as the text "None", and
/// saving the form then stored "None" as the customer's details.
#[tokio::test]
async fn empty_details_stay_empty_and_never_read_none() {
    let app = TestApp::with_accounts().await;
    let mut browser = signed_in(&app, "admin@piladerflice.cz").await;
    let edit = browser.get("/muj-ucet/upravit").await;
    edit.assert_lacks(r#"value="None""#);
    browser
        .post("/cart_items", &[("catalog_variant_id", "10"), ("quantity", "1")])
        .await;
    browser.get("/kosik/doprava").await;
    browser
        .post(
            "/kosik/doprava",
            &[("shipping_method", "pickup"), ("payment_method", "cash")],
        )
        .await;
    browser.get("/kosik/udaje").await.assert_lacks(r#"value="None""#);
    browser.get("/muj-ucet").await.assert_lacks("None");
}

#[tokio::test]
async fn checkout_is_filled_from_the_account_and_saved_back() {
    let app = TestApp::with_accounts().await;
    let mut browser = signed_in(&app, CUSTOMER).await;
    browser.get("/eshop/smrk").await;
    browser
        .post("/cart_items", &[("catalog_variant_id", "10"), ("quantity", "1")])
        .await;
    browser.get("/kosik/doprava").await;
    browser
        .post(
            "/kosik/doprava",
            &[("shipping_method", "pickup"), ("payment_method", "cash")],
        )
        .await;
    let page = browser.get("/kosik/udaje").await;
    for value in ["Karel", "Dvořák", CUSTOMER, "Hauptplatz 1", "Retz", "2070"] {
        page.assert_contains(&format!(r#"value="{value}""#));
    }

    let page = browser
        .post(
            "/kosik/udaje",
            &[
                ("first_name", "Karel"),
                ("last_name", "Dvořák"),
                ("email", CUSTOMER),
                ("phone", "+43 660 1234"),
                ("billing_street", "Neue Gasse 5"),
                ("billing_city", "Retz"),
                ("billing_zip", "2070"),
                ("billing_country", "at"),
            ],
        )
        .await;
    assert_eq!(page.location(), "/kosik/souhrn");
    browser.get("/kosik/souhrn").await;
    let page = browser.post("/kosik/souhrn", &[("terms_agreement", "1")]).await;
    assert_eq!(page.location(), "/kosik/dekujeme");

    let (user_id,): (Option<i64>,) = sqlx::query_as("SELECT user_id FROM orders ORDER BY id DESC LIMIT 1")
        .fetch_one(&app.state.db)
        .await
        .unwrap();
    assert_eq!(user_id, Some(1));
    let street: String = sqlx::query_scalar("SELECT billing_street FROM users WHERE id = 1")
        .fetch_one(&app.state.db)
        .await
        .unwrap();
    assert_eq!(street, "Neue Gasse 5");
}

#[tokio::test]
async fn past_orders_can_be_ordered_again_by_their_owner_only() {
    let app = TestApp::with_accounts().await;
    let mut browser = signed_in(&app, CUSTOMER).await;
    let page = browser.get("/muj-ucet").await;
    page.assert_contains("/muj-ucet/objednavky/500/znovu-objednat");

    // Somebody else's order.
    let page = browser.post("/muj-ucet/objednavky/501/znovu-objednat", &[]).await;
    assert_eq!(page.status, StatusCode::NOT_FOUND);

    // One of the two items is no longer sold.
    let page = browser.post("/muj-ucet/objednavky/500/znovu-objednat", &[]).await;
    assert_eq!(page.location(), "/kosik");
    let cart = browser.get("/kosik").await;
    cart.assert_contains("1 položek z této objednávky už není dostupných");
    cart.assert_contains("Smrk, Sypané, 25 cm, 5 PRM");
    cart.assert_contains(r#"<input type="number" name="quantity" value="2""#);
}

#[tokio::test]
async fn the_picture_is_cleaned_shown_only_to_its_owner_and_removable() {
    let app = TestApp::with_accounts().await;
    let mut browser = signed_in(&app, CUSTOMER).await;
    browser.get("/muj-ucet/upravit").await;

    // Not a picture.
    let page = browser
        .post_file("/muj-ucet/profilovy-obrazek", &[], ("avatar", "x.gif", b"GIF89a....."))
        .await;
    assert_eq!(page.location(), "/muj-ucet/upravit");
    browser
        .get("/muj-ucet/upravit")
        .await
        .assert_contains("Podporujeme pouze obrázky JPG, PNG a WebP.");

    let page = browser
        .post_file("/muj-ucet/profilovy-obrazek", &[], ("avatar", "ja.png", &png()))
        .await;
    assert_eq!(page.location(), "/muj-ucet/upravit");
    let edit = browser.get("/muj-ucet/upravit").await;
    edit.assert_contains("Profilový obrázek byl uložen.");
    edit.assert_contains(r#"class="account-btn-avatar""#);
    edit.assert_contains("/muj-ucet/profilovy-obrazek?v=");

    let picture = browser.get("/muj-ucet/profilovy-obrazek").await;
    assert_eq!(picture.status, StatusCode::OK);
    assert_eq!(picture.headers[header::CONTENT_TYPE], "image/webp");
    assert_eq!(
        picture.headers[header::CONTENT_SECURITY_POLICY],
        "default-src 'none'; sandbox"
    );
    assert!(picture.body.starts_with("RIFF") && picture.body[8..12] == *"WEBP");

    // Other visitors only get sent to the sign-in page.
    let page = app.browser().get("/muj-ucet/profilovy-obrazek").await;
    assert_eq!(page.location(), "/prihlaseni");

    let page = browser.post("/muj-ucet/profilovy-obrazek/smazat", &[]).await;
    assert_eq!(page.location(), "/muj-ucet/upravit");
    browser
        .get("/muj-ucet/upravit")
        .await
        .assert_contains("Profilový obrázek byl odstraněn.");
    assert_eq!(
        browser.get("/muj-ucet/profilovy-obrazek").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        files_in(&app.state.config.storage_root),
        1,
        "only the fixture picture is left"
    );

    // At most 10 uploads per 10 minutes (two were made above).
    let mut last = None;
    for _ in 0..9 {
        browser
            .post_file("/muj-ucet/profilovy-obrazek", &[], ("avatar", "x.gif", b"GIF"))
            .await;
        last = Some(browser.get("/muj-ucet/upravit").await);
    }
    last.unwrap().assert_contains("Příliš mnoho pokusů o nahrání.");
}
