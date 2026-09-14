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

  test "kontakt renders the kalkulace form with both category field sets" do
    get kontakt_url
    assert_response :success
    assert_select "fieldset.kalkulace-fields-section", count: InquiryFormOption::CATEGORIES.size
    assert_select "select[name='varianta[]'] option", text: inquiry_form_options(:palivove_varianta_skladane).value
    assert_select "select[name='polozka[]'] option", text: inquiry_form_options(:stavebni_polozka_tram).value
  end
end
