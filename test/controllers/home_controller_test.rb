require "test_helper"

class HomeControllerTest < ActionDispatch::IntegrationTest
  test "should get index" do
    get root_url
    assert_response :success
  end

  test "eshop defaults to the palivove category" do
    get eshop_url
    assert_response :success
    assert_select ".product-card", count: CatalogProduct.active.where(category: "palivove").count
  end

  test "eshop filters by category" do
    get eshop_url(category: "rezivo")
    assert_response :success
    assert_select ".product-card", count: CatalogProduct.active.where(category: "rezivo").count
  end

  test "eshop excludes inactive products" do
    get eshop_url(category: "palivove")
    assert_response :success
    assert_select "h4.product-title", text: catalog_products(:inactive_product).title, count: 0
  end

  test "eshop filters firewood by wood hardness" do
    get eshop_url(category: "palivove", wood: "mekke")
    assert_response :success
    assert_select "h4.product-title", text: catalog_products(:smrk).title
  end

  test "product pages render every display template" do
    %i[smrk tramy odkory].each do |name|
      product = catalog_products(name)
      get eshop_product_url(product.key)
      assert_response :success
      assert_select "h1", text: product.title
    end
  end

  test "product page breadcrumb links to the product's e-shop category" do
    get eshop_product_url(catalog_products(:tramy).key)
    assert_select "a.breadcrumb-item[href='/eshop?category=rezivo']", text: "Stavební řezivo"
  end

  test "icon-only add-to-cart button keeps an accessible label" do
    get eshop_product_url(catalog_products(:tramy).key)
    assert_select "form.add-to-cart-form button[type=submit][aria-label=?][title=?] svg", "Do košíku", "Do košíku"
  end

  test "product created in the admin gets a detail page without code changes" do
    product = CatalogProduct.create!(key: "prkna-iii", template: "lumber", category: "rezivo", title: "Prkna III")
    product.catalog_variants.create!(key: "iii-3000", grade: "III. jakostní třída", length_label: "3 000 mm", price_czk: 4_000)

    get eshop_product_url(product.key)
    assert_response :success
    assert_select "h1", text: "Prkna III"
    assert_select ".spec-value", text: "III. jakostní třída"
  end

  test "inactive and unknown products are not found" do
    get eshop_product_url(catalog_products(:inactive_product).key)
    assert_response :not_found

    get eshop_product_url("neexistuje")
    assert_response :not_found
  end

  test "kontakt ceník follows the admin order and only keeps formatting tags" do
    pricelist_items(:tramy).update!(position: -1, price: "9 800 Kč / m<sup>3</sup><script>alert(1)</script>")

    get kontakt_url
    assert_select "#cenik .cenik-group:first-of-type tr:first-child td:first-child", text: "Trámy"
    assert_select "#cenik sup", minimum: 1
    assert_select "#cenik script", count: 0
  end

  test "kontakt renders the kalkulace form with both category field sets" do
    get kontakt_url
    assert_response :success
    assert_select "fieldset.kalkulace-fields-section", count: InquiryFormOption::CATEGORIES.size
    assert_select "select[name='varianta[]'] option", text: inquiry_form_options(:palivove_varianta_skladane).value
    assert_select "select[name='polozka[]'] option", text: inquiry_form_options(:stavebni_polozka_tram).value
  end

  test "kontakt only offers the custom option on palivove's mnozstvi field" do
    get kontakt_url
    assert_response :success

    assert_select "select[name='varianta[]'] option[value='__custom__']", count: 0
    assert_select "select[name='druh[]'] option[value='__custom__']", count: 0
    assert_select "fieldset[data-category='palivove'] select[name='delka[]'] option[value='__custom__']", count: 0
    assert_select "select[name='mnozstvi[]'] option[value='__custom__']", count: 1

    assert_select "select[name='polozka[]'] option[value='__custom__']", count: 1
    assert_select "fieldset[data-category='stavebni'] select[name='delka[]'] option[value='__custom__']", count: 1
    assert_select "select[name='vyska[]'] option[value='__custom__']", count: 1
    assert_select "select[name='sirka[]'] option[value='__custom__']", count: 1
  end

  test "phone numbers and e-mail are clickable in header, footer and mobile sidebar" do
    get root_url
    assert_select ".contact-header a[href=?]", "tel:+420602446339"
    assert_select ".contact-header a[href=?]", "mailto:stepan.merta@seznam.cz"
    assert_select ".sidebar-contact a[href=?]", "tel:+420602446339"
    assert_select ".sidebar-contact a[href=?]", "mailto:stepan.merta@seznam.cz"
    assert_select "footer a[href=?]", "tel:+420602446339"
    assert_select "footer a[href=?]", "tel:+420515235527"
    assert_select "footer a[href=?]", "mailto:stepan.merta@seznam.cz"
  end

  test "contact page phone numbers are tel links" do
    get kontakt_url
    assert_select ".kontakt-list-items a[href=?]", "tel:+420602446339"
    assert_select ".kontakt-list-items a[href=?]", "tel:+420515235527"
  end
end
