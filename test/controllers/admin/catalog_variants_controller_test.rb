require "test_helper"

class Admin::CatalogVariantsControllerTest < ActionDispatch::IntegrationTest
  setup do
    post session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }
    @product = catalog_products(:smrk)
    @variant = catalog_variants(:smrk_kontejner_100cm)
  end

  test "the form for firewood has the container/crates choice and the stacked checkbox" do
    get edit_admin_catalog_product_catalog_variant_path(@product, @variant)

    assert_response :success
    assert_select "select#catalog_variant_variant_group option", text: "Kontejneru"
    assert_select "select#catalog_variant_variant_group option", text: "Bednách"
    assert_select "input[type=checkbox][name='catalog_variant[stacked]']"
  end

  test "ticking 'stacked' saves it, and unticking turns it off again" do
    patch admin_catalog_product_catalog_variant_path(@product, @variant), params: { catalog_variant: { stacked: "1" } }
    assert @variant.reload.stacked

    patch admin_catalog_product_catalog_variant_path(@product, @variant), params: { catalog_variant: { stacked: "0" } }
    assert_not @variant.reload.stacked
  end

  test "the product's variant list tells loose from stacked container wood" do
    @variant.update!(stacked: true)
    get edit_admin_catalog_product_path(@product)

    assert_match "Skládané (kontejner)", response.body
    assert_match "Skládané (bedny)", response.body
  end
end
