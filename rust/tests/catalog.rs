//! The catalogue pages (Phase 2): home, e-shop, product pages, contact,
//! services, Sortiment and the sitemap, against the fixture data.

mod common;

use axum::http::{StatusCode, header};
use common::{TestApp, get};

#[tokio::test]
async fn home_shows_promo_bestsellers_services_and_reviews() {
    let page = get("/").await;
    assert_eq!(page.status, StatusCode::OK);
    page.assert_contains("<title>Pila Derflice – palivové dřevo a řezivo u Znojma</title>");
    page.assert_contains(r#"<h2 class="product-title">Akční smrk</h2>"#);
    page.assert_contains(r#"<span class="akcni-nabidka-price">od 8 500 Kč</span>"#);
    page.assert_contains(r#"<span class="nejprodavanejsi-produkty-desc"><strong>Dub</strong></span>"#);
    page.assert_contains("od 11 000 Kč");
    page.assert_contains(r#"<a class="btn-cenik" href="/sluzby#doprava">Ceník</a>"#);
    // Reviews are typed text: escaped.
    page.assert_contains("Skvělé dřevo &lt;3");
    page.assert_contains(r#"<div class="google-review-avatar" style="background-color: #EA4335;">J</div>"#);
    page.assert_contains(r#"<script type="application/ld+json">{"@context":"https://schema.org""#);
    page.assert_contains(r#""@id":"http://piladerflice.test/#business""#);
}

#[tokio::test]
async fn austrian_home_converts_prices_to_euros() {
    let page = get("/at").await;
    page.assert_contains(r#"<span class="akcni-nabidka-price">od ≈ 340,00 €</span>"#);
    page.assert_contains(r#"<h2 class="product-title">Fichte im Angebot</h2>"#);
    page.assert_contains("≈ 440,00 €");
    page.assert_contains(r#"<a class="akce-btn-eshop" href="/at/eshop/smrk">"#);
    page.assert_contains(r#""@id":"http://piladerflice.test/at#business""#);
}

#[tokio::test]
async fn eshop_lists_a_category_with_badges_and_from_prices() {
    let page = get("/eshop").await;
    page.assert_contains("<title>Palivové dřevo – e-shop | Pila Derflice</title>");
    page.assert_contains(r#"<span class="product-badge">1.</span>"#);
    // Smrk's cheapest variant in stock.
    page.assert_contains("od 2 500 Kč");
    page.assert_lacks("Fošny");
    page.assert_lacks("Skrytý");
    page.assert_contains(
        r#"<img alt="Smrk" class="product-img" src="/uploads/testkey0000000000000000000001/smrk%20foto.webp">"#,
    );

    let hard = get("/eshop?category=palivove&wood=tvrde").await;
    hard.assert_lacks(r#"<h4 class="product-title">Smrk</h4>"#);
    hard.assert_contains(r#"<h4 class="product-title">Dub</h4>"#);

    let lumber = get("/eshop?category=rezivo").await;
    lumber.assert_contains(r#"<h4 class="product-title">Fošny</h4>"#);
    lumber.assert_contains(r#"<link rel="canonical" href="http://piladerflice.test/eshop?category=rezivo">"#);
}

#[tokio::test]
async fn firewood_page_has_the_container_grid_and_crates() {
    let page = get("/eshop/smrk").await;
    page.assert_contains("<title>Smrk – Palivové dřevo | Pila Derflice</title>");
    // strip_tags for the description
    page.assert_contains(r#"<meta name="description" content="Suché smrkové dřevo.">"#);
    page.assert_contains(r#"<div class="matrix-cell col-header">25 cm</div>"#);
    page.assert_contains(r#"<div class="matrix-cell row-header"><span>13 a 14 PRM</span></div>"#);
    page.assert_contains("21 500 Kč");
    // 13 PRM at 50 cm is sold out
    page.assert_contains(r#"disabled="disabled""#);
    page.assert_contains(r#"<h2 class="section-title">"#);
    page.assert_contains(r##"<a class="breadcrumb-item active" href="#">Smrk</a>"##);
    page.assert_contains(r#"<a class="breadcrumb-item " href="/eshop?category=palivove">Palivové dřevo</a>"#);
    assert_eq!(page.body.matches(r#"class="add-to-cart-form""#).count(), 4);
}

#[tokio::test]
async fn lumber_page_filters_by_dimension() {
    let page = get("/at/eshop/fosny").await;
    page.assert_contains(r#"<button type="button" class="filter-btn" data-filter="200x50">200×50 mm</button>"#);
    page.assert_contains(r#"<div class="inventory-card" data-filter-value="240x60">"#);
    page.assert_contains(r#"<span class="spec-value">Qualität AB</span>"#);
    page.assert_contains("≈ 356,00 €");
}

#[tokio::test]
async fn simple_variant_page_marks_sold_out_variants() {
    let page = get("/eshop/stepka").await;
    page.assert_contains(r#"<div class="simple-grid-2">"#);
    page.assert_contains(r#"<span class="spec-value">Kontejner 5 m³</span>"#);
    page.assert_contains(r#"<p class="spec-value text-red""#);
    // Without a description, the meta description says what the product is.
    page.assert_contains(r#"<meta name="description" content=""#);
}

#[tokio::test]
async fn hidden_and_unknown_products_are_not_found() {
    assert_eq!(get("/eshop/skryty").await.status, StatusCode::NOT_FOUND);
    assert_eq!(get("/at/eshop/nic").await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn contact_page_shows_the_price_list_safely() {
    let page = get("/kontakt").await;
    page.assert_contains("<td>9 800 Kč / m<sup>3</sup></td>");
    page.assert_contains(r#"<h4 class="cenik-subtitle">Měkké</h4>"#);
    page.assert_contains("<td>1 800 Kč / PRM</td>");
    // "---" has no unit after it
    page.assert_contains("<td>---</td>");
    // typed HTML is cleaned
    page.assert_lacks("<script>x</script>");
    page.assert_contains(r#"<option value="Buk">Buk</option>"#);
    page.assert_contains(r#"<fieldset class="kalkulace-fields-section" data-category="stavebni" hidden disabled>"#);
    page.assert_contains(r#"<a target="_blank" rel="noopener" href="/zasady-ochrany-osobnich-udaju">"#);

    let german = get("/at/kontakt").await;
    german.assert_contains("<td>Kanthölzer</td>");
    german.assert_contains("<td>≈ 392,00 € / m<sup>3</sup></td>");
    // firewood has its own German prices
    german.assert_contains("<td>75 € / Rm</td>");
    german.assert_contains(r#"<option value="Buche">Buche</option>"#);
    german.assert_contains(r#"href="/at/zasady-ochrany-osobnich-udaju""#);
}

#[tokio::test]
async fn services_page_has_anchors_paragraphs_and_carousels() {
    let page = get("/sluzby").await;
    page.assert_contains(r#"<div class="sluzba-row " id="doprava">"#);
    page.assert_contains(r#"<div class="sluzba-row reverse-layout" id="/sluzby">"#);
    page.assert_contains("<p>Dovezeme.<br><br>Vlastním vozem.</p>");
    page.assert_contains(r#"<li class="carousel-slide active">"#);
    page.assert_lacks("Cenová kalkulace");
}

#[tokio::test]
async fn sortiment_lists_the_categories() {
    let page = get("/at/sortiment").await;
    page.assert_contains(r#"<a class="sortiment-card" href="/at/sortiment/palivove-drevo">"#);
    page.assert_contains("<h3>Brennholz</h3>");
}

#[tokio::test]
async fn sitemap_lists_both_languages() {
    let page = get("/sitemap.xml").await;
    assert_eq!(page.headers[header::CONTENT_TYPE], "application/xml; charset=utf-8");
    page.assert_contains("<loc>http://piladerflice.test/at/eshop/smrk</loc>");
    page.assert_contains("<lastmod>2026-03-05T08:30:00Z</lastmod>");
    page.assert_contains(r#"hreflang="de-AT" href="http://piladerflice.test/at/eshop?category=rezivo""#);
    page.assert_lacks("skryty");
    page.assert_lacks("category=kamenivo");
}

#[tokio::test]
async fn uploaded_images_are_served_with_safe_headers() {
    let app = TestApp::new().await;
    let image = app.get("/uploads/testkey0000000000000000000001/smrk%20foto.webp").await;
    assert_eq!(image.status, StatusCode::OK);
    assert_eq!(image.headers[header::CONTENT_TYPE], "image/webp");
    assert_eq!(image.headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    assert_eq!(
        app.get("/uploads/unknownkey/x.webp").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.get("/uploads/..%2F..%2Fsecret/x").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn pages_work_with_an_empty_database() {
    let app = TestApp::empty().await;
    for path in ["/", "/kontakt", "/sluzby", "/sortiment", "/eshop", "/sitemap.xml"] {
        assert_eq!(app.get(path).await.status, StatusCode::OK, "{path}");
    }
}
