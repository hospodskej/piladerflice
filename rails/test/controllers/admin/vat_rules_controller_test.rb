require "test_helper"

class Admin::VatRulesControllerTest < ActionDispatch::IntegrationTest
  setup do
    post session_path, params: { email_address: users(:admin).email_address, password: "correct-horse-battery" }
  end

  test "lists every category with its rates" do
    get admin_vat_rules_path
    assert_response :success
    assert_select "td strong", text: "Stavební řezivo"
    assert_select "input[name='vat_rule[rate_at]'][value='13']", count: 2
  end

  test "a changed rate applies to what customers pay" do
    patch admin_vat_rule_path("rezivo"), params: { vat_rule: { prices_include_vat: "false", rate_cz: "15", rate_at: "20" } }
    assert_redirected_to admin_vat_rules_path

    assert_equal 9_775, catalog_variants(:tramy_variant).reload.customer_price_czk # 8 500 + 15 %
  end

  test "a rate out of range is refused" do
    patch admin_vat_rule_path("rezivo"), params: { vat_rule: { rate_cz: "150", rate_at: "20" } }
    assert_response :unprocessable_entity
    assert_equal 21, vat_rules(:rezivo).reload.rate_cz
  end

  test "the price field says whether the category's prices include VAT" do
    variant = catalog_variants(:tramy_variant)
    get edit_admin_catalog_product_catalog_variant_path(variant.catalog_product, variant)
    assert_select "label[for=catalog_variant_price_czk]", text: "Cena (Kč, bez DPH)"
  end
end
