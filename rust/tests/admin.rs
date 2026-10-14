//! The admin area (Phase 5): who gets in, and each section's pages and
//! forms. The admin account is in `fixtures/accounts.sql`.

mod common;

use axum::http::StatusCode;
use common::{Browser, TestApp};

const ADMIN: &str = "admin@piladerflice.cz";

/// A small picture to upload, as PNG.
fn png() -> Vec<u8> {
    let picture = image::RgbImage::from_fn(30, 20, |x, y| image::Rgb([(x * 8) as u8, (y * 12) as u8, 60]));
    let mut bytes = std::io::Cursor::new(Vec::new());
    picture.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
    bytes.into_inner()
}

async fn admin(app: &TestApp) -> Browser<'_> {
    let mut browser = app.browser();
    assert_eq!(browser.sign_in(ADMIN).await.location(), "/muj-ucet");
    browser
}

async fn count(app: &TestApp, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(&app.state.db).await.unwrap()
}

#[tokio::test]
async fn only_admins_get_in() {
    let app = TestApp::with_accounts().await;

    // Not signed in: to the sign-in page, and back here afterwards.
    let mut visitor = app.browser();
    let page = visitor.get("/admin/orders").await;
    assert_eq!((page.status, page.location()), (StatusCode::SEE_OTHER, "/prihlaseni"));
    visitor.get("/prihlaseni").await;
    let page = visitor
        .post("/prihlaseni", &[("email_address", ADMIN), ("password", "heslo12345")])
        .await;
    assert_eq!(page.location(), "/admin/orders");

    // A customer: sent home, and can't change anything either.
    let mut customer = app.browser();
    customer.sign_in("zakaznik@example.com").await;
    let page = customer.get("/admin").await;
    assert_eq!(page.location(), "/");
    customer
        .get("/")
        .await
        .assert_contains("Nemáte oprávnění k přístupu do administrace.");
    let page = customer.post("/admin/orders/500", &[("_method", "delete")]).await;
    assert_eq!(page.location(), "/");
    assert_eq!(count(&app, "SELECT COUNT(*) FROM orders WHERE id = 500").await, 1);

    // The admin is Czech only.
    assert_eq!(admin(&app).await.get("/at/admin").await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn forms_without_the_token_are_refused() {
    let app = TestApp::with_accounts().await;
    let mut browser = admin(&app).await;
    let page = browser
        .post_raw("/admin/orders/500", &[("_method", "delete".into())])
        .await;
    assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(count(&app, "SELECT COUNT(*) FROM orders").await, 2);
}

#[tokio::test]
async fn the_overview_counts_and_lists_the_latest_orders() {
    let app = TestApp::with_accounts().await;
    let page = admin(&app).await.get("/admin").await;
    assert_eq!(page.status, StatusCode::OK);
    page.assert_contains(r#"<span class="admin-dashboard-card-number">5</span>"#);
    page.assert_contains(r#"<a href="/admin/orders/501">#501</a>"#);
    page.assert_contains("<td>18 100 Kč</td>");
    page.assert_contains("<td>01.05.2026 10:00</td>");
    page.assert_contains(r#"<a class="admin-nav-link active" href="/admin">Přehled</a>"#);
    page.assert_contains(r#"<span class="admin-current-user">admin@piladerflice.cz</span>"#);
}

#[tokio::test]
async fn an_order_is_shown_in_full_and_can_be_deleted() {
    let app = TestApp::with_accounts().await;
    let mut browser = admin(&app).await;
    let page = browser.get("/admin/orders/500").await;
    page.assert_contains("Karel Dvořák");
    page.assert_contains("Rakousko");
    page.assert_contains("<td>Smrk</td>");
    // A product no longer sold still gets a readable name.
    page.assert_contains("<td>Zmizely</td>");
    page.assert_contains("Osobní odběr na adrese skladu");
    page.assert_contains("<strong>Celkem k úhradě: 18 100 Kč</strong>");

    assert_eq!(browser.get("/admin/orders/9999").await.status, StatusCode::NOT_FOUND);
    assert_eq!(browser.get("/admin/orders/abc").await.status, StatusCode::NOT_FOUND);

    let page = browser.post("/admin/orders/500", &[("_method", "delete")]).await;
    assert_eq!(page.location(), "/admin/orders");
    let list = browser.get("/admin/orders").await;
    list.assert_contains("Objednávka byla smazána.");
    list.assert_lacks("/admin/orders/500");
}

#[tokio::test]
async fn quote_requests_are_listed_with_their_items() {
    let app = TestApp::with_accounts().await;
    sqlx::query(
        "INSERT INTO inquiries (id, category, first_name, last_name, email, phone, street, city, notes, items_snapshot,
           locale, created_at, updated_at)
         VALUES (7, 'palivove', 'Eva', 'Malá', 'eva@example.com', '777', 'Lesní 3', 'Znojmo', 'Rychle prosím',
           '[{\"varianta\":\"Sypané\",\"druh\":\"Buk\",\"delka\":\"33 cm\",\"mnozstvi\":5}]', 'de', '2026-06-01 08:00:00', '2026-06-01 08:00:00')",
    )
    .execute(&app.state.db)
    .await
    .unwrap();
    let mut browser = admin(&app).await;
    let list = browser.get("/admin/inquiries").await;
    list.assert_contains("<td>Palivové dřevo</td>");
    let page = browser.get("/admin/inquiries/7").await;
    page.assert_contains("jazyk formuláře: němčina");
    page.assert_contains("<p>Lesní 3, Znojmo</p>");
    page.assert_contains("<th>Množství</th>");
    page.assert_contains("<td>5</td>");
    page.assert_contains("Rychle prosím");
}

#[tokio::test]
async fn a_product_is_checked_created_and_edited() {
    let app = TestApp::with_accounts().await;
    let mut browser = admin(&app).await;
    assert_eq!(browser.get("/admin/catalog_products/new").await.status, StatusCode::OK);

    let page = browser
        .post(
            "/admin/catalog_products",
            &[
                ("catalog_product[key]", "smrk"),
                ("catalog_product[template]", "lumber"),
                ("catalog_product[category]", "x"),
            ],
        )
        .await;
    assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);
    page.assert_contains("2 chyby brání uložení:");
    page.assert_contains("<li>Klíč už je použitý</li>");
    page.assert_contains("<li>Kategorie filtru má neplatnou hodnotu</li>");
    page.assert_contains(
        r#"<div class="field_with_errors"><input type="text" value="smrk" name="catalog_product[key]""#,
    );

    let page = browser
        .post_file(
            "/admin/catalog_products",
            &[
                ("catalog_product[key]", "tramy"),
                ("catalog_product[template]", "lumber"),
                ("catalog_product[category]", "rezivo"),
                ("catalog_product[hardness]", ""),
                ("catalog_product[active]", "0"),
                ("catalog_product[position]", ""),
                ("catalog_product[title]", "Trámy"),
                ("catalog_product[image_alt_de]", "Balken"),
            ],
            ("catalog_product[image]", "trámy.png", &png()),
        )
        .await;
    let id: i64 = count(&app, "SELECT id FROM catalog_products WHERE key = 'tramy'").await;
    assert_eq!(page.location(), format!("/admin/catalog_products/{id}/edit"));
    let row: (Option<String>, bool, i64, Option<String>) =
        sqlx::query_as("SELECT hardness, active, position, image_alt_de FROM catalog_products WHERE id = ?")
            .bind(id)
            .fetch_one(&app.state.db)
            .await
            .unwrap();
    assert_eq!(row, (None, false, 0, Some("Balken".into())));
    let blob: (String, String, String) = sqlx::query_as(
        "SELECT b.filename, b.content_type, b.metadata FROM active_storage_attachments a
         JOIN active_storage_blobs b ON b.id = a.blob_id WHERE a.record_type = 'CatalogProduct' AND a.record_id = ?",
    )
    .bind(id)
    .fetch_one(&app.state.db)
    .await
    .unwrap();
    assert_eq!(
        blob,
        (
            "trámy.png".into(),
            "image/png".into(),
            r#"{"identified":true,"width":30,"height":20,"analyzed":true}"#.into()
        )
    );

    let page = browser.get(&format!("/admin/catalog_products/{id}/edit")).await;
    page.assert_contains("Produkt byl vytvořen. Nyní k němu přidejte varianty s cenami.");
    page.assert_contains(r#"<img class="admin-image-preview" src="/uploads/"#);
    page.assert_contains("Tento produkt zatím nemá žádné varianty");

    let page = browser
        .post(
            &format!("/admin/catalog_products/{id}"),
            &[("_method", "patch"), ("catalog_product[title]", "Trámy na míru")],
        )
        .await;
    assert_eq!(page.location(), format!("/admin/catalog_products/{id}/edit"));
    let page = browser.get(&format!("/admin/catalog_products/{id}/edit")).await;
    page.assert_contains("Upravit produkt: Trámy na míru");
    // The picture stays when no new one is chosen.
    page.assert_contains(r#"<img class="admin-image-preview""#);
}

#[tokio::test]
async fn only_pictures_can_be_uploaded() {
    let app = TestApp::with_accounts().await;
    let mut browser = admin(&app).await;
    let page = browser
        .post_file(
            "/admin/promos",
            &[("promo[title]", "X")],
            ("promo[image]", "x.svg", b"<svg onload=alert(1)>"),
        )
        .await;
    assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);
    page.assert_contains("Obrázek musí být fotka ve formátu JPG, PNG nebo WebP.");
    assert_eq!(count(&app, "SELECT COUNT(*) FROM promos").await, 1);
}

#[tokio::test]
async fn deleting_a_product_takes_its_variants_and_picture_along() {
    let app = TestApp::with_accounts().await;
    let mut browser = admin(&app).await;
    let file = app
        .state
        .config
        .storage_root
        .join("te/st/testkey0000000000000000000001");
    assert!(file.exists());
    let page = browser
        .post("/admin/catalog_products/1", &[("_method", "delete")])
        .await;
    assert_eq!(page.location(), "/admin/catalog_products");
    browser
        .get("/admin/catalog_products")
        .await
        .assert_contains("Produkt byl smazán.");
    assert_eq!(
        count(
            &app,
            "SELECT COUNT(*) FROM catalog_variants WHERE catalog_product_id = 1"
        )
        .await,
        0
    );
    assert_eq!(
        count(&app, "SELECT COUNT(*) FROM active_storage_blobs WHERE id = 1").await,
        0
    );
    assert!(!file.exists());
}

#[tokio::test]
async fn variants_are_checked_and_belong_to_their_product() {
    let app = TestApp::with_accounts().await;
    let mut browser = admin(&app).await;
    // Variant 10 is the spruce's, not the oak's.
    let page = browser.get("/admin/catalog_products/2/catalog_variants/10/edit").await;
    assert_eq!(page.status, StatusCode::NOT_FOUND);
    // A lumber product's form asks for sizes.
    browser
        .get("/admin/catalog_products/3/catalog_variants/new")
        .await
        .assert_contains(r#"id="catalog_variant_width_mm""#);

    let page = browser
        .post(
            "/admin/catalog_products/1/catalog_variants",
            &[("catalog_variant[key]", "k-25-5"), ("catalog_variant[price_czk]", "0")],
        )
        .await;
    assert_eq!(page.status, StatusCode::UNPROCESSABLE_ENTITY);
    page.assert_contains("<li>Klíč už je použitý</li>");
    page.assert_contains("<li>Cena musí být větší než 0</li>");

    // An emptied position saves as 0 (Rails failed here).
    let page = browser
        .post(
            "/admin/catalog_products/1/catalog_variants",
            &[
                ("catalog_variant[key]", "k-100-5"),
                ("catalog_variant[price_czk]", "12000"),
                ("catalog_variant[variant_group]", "bedny"),
                ("catalog_variant[position]", ""),
                ("catalog_variant[in_stock]", "0"),
            ],
        )
        .await;
    assert_eq!(page.location(), "/admin/catalog_products/1/edit");
    let row: (i64, bool, i64) = sqlx::query_as(
        "SELECT position, in_stock, price_czk FROM catalog_variants WHERE key = 'k-100-5' AND catalog_product_id = 1",
    )
    .fetch_one(&app.state.db)
    .await
    .unwrap();
    assert_eq!(row, (0, false, 12000));
    let page = browser.get("/admin/catalog_products/1/edit").await;
    page.assert_contains("Varianta byla přidána.");
    page.assert_contains("Skládané (bedny)");
}

#[tokio::test]
async fn promos_are_added_edited_and_deleted() {
    let app = TestApp::with_accounts().await;
    let mut browser = admin(&app).await;
    browser.get("/admin/promos").await.assert_contains("<td>✓ Aktivní</td>");
    let page = browser
        .post(
            "/admin/promos",
            &[("promo[title]", "Dubové trámy"), ("promo[price]", "od 9 000 Kč")],
        )
        .await;
    assert_eq!(page.location(), "/admin/promos");
    let list = browser.get("/admin/promos").await;
    list.assert_contains("Doporučení bylo vytvořeno.");
    list.assert_contains("<strong>Dubové trámy</strong>");
    let id = count(&app, "SELECT MAX(id) FROM promos").await;
    browser
        .post(
            &format!("/admin/promos/{id}"),
            &[("_method", "patch"), ("promo[title_de]", "Eichenbalken")],
        )
        .await;
    assert_eq!(
        count(&app, "SELECT COUNT(*) FROM promos WHERE title_de = 'Eichenbalken'").await,
        1
    );
    browser
        .post(&format!("/admin/promos/{id}"), &[("_method", "delete")])
        .await;
    assert_eq!(count(&app, "SELECT COUNT(*) FROM promos").await, 1);
}

#[tokio::test]
async fn the_price_list_is_edited_row_by_row() {
    let app = TestApp::with_accounts().await;
    let mut browser = admin(&app).await;
    let page = browser
        .post(
            "/admin/pricelist_items",
            &[("pricelist_item[category]", "palivove_volne")],
        )
        .await;
    assert_eq!(page.location(), "/admin/pricelist_items#palivove_volne");
    browser
        .get("/admin/pricelist_items")
        .await
        .assert_contains("Cena je povinné pole, Název je povinné pole, Dřevina je povinné pole");

    let page = browser
        .post(
            "/admin/pricelist_items",
            &[
                ("pricelist_item[category]", "palivove_volne"),
                ("pricelist_item[subcategory]", "Tvrdé"),
                ("pricelist_item[item_name]", "50 cm"),
                ("pricelist_item[price]", "2 400 Kč"),
            ],
        )
        .await;
    assert_eq!(page.location(), "/admin/pricelist_items#palivove_volne");
    let new_id = count(&app, "SELECT MAX(id) FROM pricelist_items").await;
    assert_eq!(
        count(
            &app,
            &format!("SELECT position FROM pricelist_items WHERE id = {new_id}")
        )
        .await,
        2
    );

    // Moving up numbers the category afresh; the category itself can't change.
    browser
        .post(
            &format!("/admin/pricelist_items/{new_id}/move_up"),
            &[("_method", "patch")],
        )
        .await;
    let order: Vec<i64> =
        sqlx::query_scalar("SELECT id FROM pricelist_items WHERE category = 'palivove_volne' ORDER BY position, id")
            .fetch_all(&app.state.db)
            .await
            .unwrap();
    assert_eq!(order, [2, new_id, 3]);
    browser
        .post(
            &format!("/admin/pricelist_items/{new_id}"),
            &[
                ("_method", "patch"),
                ("pricelist_item[category]", "zbytky"),
                ("pricelist_item[price_de]", "96 €"),
            ],
        )
        .await;
    assert_eq!(
        count(&app, &format!("SELECT COUNT(*) FROM pricelist_items WHERE id = {new_id} AND category = 'palivove_volne' AND price_de = '96 €'")).await,
        1
    );
    // Kamenivo is a single price: no row for adding another.
    browser
        .get("/admin/pricelist_items")
        .await
        .assert_lacks("new-pricelist-item-kamenivo-form");
}

#[tokio::test]
async fn quote_form_choices_are_added_checked_and_moved() {
    let app = TestApp::with_accounts().await;
    let mut browser = admin(&app).await;
    let page = browser
        .post(
            "/admin/inquiry_form_options",
            &[
                ("inquiry_form_option[category]", "palivove"),
                ("inquiry_form_option[field]", "polozka"),
                ("inquiry_form_option[value]", ""),
            ],
        )
        .await;
    assert_eq!(page.location(), "/admin/inquiry_form_options");
    browser
        .get("/admin/inquiry_form_options")
        .await
        .assert_contains("Hodnota je povinné pole a Pole neodpovídá zvolené kategorii");

    let page = browser
        .post(
            "/admin/inquiry_form_options",
            &[
                ("inquiry_form_option[category]", "palivove"),
                ("inquiry_form_option[field]", "druh"),
                ("inquiry_form_option[value]", "Habr"),
            ],
        )
        .await;
    assert_eq!(page.location(), "/admin/inquiry_form_options#palivove-druh");
    let habr = count(&app, "SELECT id FROM inquiry_form_options WHERE value = 'Habr'").await;
    assert_eq!(
        count(
            &app,
            &format!("SELECT position FROM inquiry_form_options WHERE id = {habr}")
        )
        .await,
        2
    );

    browser
        .post(
            &format!("/admin/inquiry_form_options/{habr}/move_up"),
            &[("_method", "patch")],
        )
        .await;
    let order: Vec<String> =
        sqlx::query_scalar("SELECT value FROM inquiry_form_options WHERE field = 'druh' ORDER BY position, id")
            .fetch_all(&app.state.db)
            .await
            .unwrap();
    assert_eq!(order, ["Buk", "Habr", "Dub"]);
    // An option stays in its list.
    browser
        .post(
            &format!("/admin/inquiry_form_options/{habr}"),
            &[
                ("_method", "patch"),
                ("inquiry_form_option[field]", "delka"),
                ("inquiry_form_option[value_de]", "Hainbuche"),
            ],
        )
        .await;
    assert_eq!(
        count(&app, &format!("SELECT COUNT(*) FROM inquiry_form_options WHERE id = {habr} AND field = 'druh' AND value_de = 'Hainbuche'")).await,
        1
    );
}
