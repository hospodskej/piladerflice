require "test_helper"

class CatalogProductTest < ActiveSupport::TestCase
  test "title_i18n falls back to the Czech title when no German translation is set" do
    product = catalog_products(:odkory)
    product.update!(title_de: nil)

    I18n.with_locale(:de) { assert_equal product.title, product.title_i18n }
  end

  test "title_i18n uses the German title when present" do
    product = catalog_products(:smrk)
    I18n.with_locale(:de) { assert_equal "Fichte", product.title_i18n }
  end

  test "starting_price_czk is the cheapest in-stock variant" do
    product = catalog_products(:smrk)
    assert_equal 900, product.starting_price_czk
  end

  test "starting_price_czk falls back to the cheapest variant when none are in stock" do
    product = catalog_products(:smrk)
    product.catalog_variants.update_all(in_stock: false)
    cheapest_overall = product.catalog_variants.reorder(:price_czk).first

    assert_equal cheapest_overall.price_czk, product.starting_price_czk
  end

  test "starting_price_czk is nil for a product with no variants" do
    product = catalog_products(:odkory)
    product.catalog_variants.destroy_all

    assert_nil product.starting_price_czk
  end

  test "requires a unique key" do
    duplicate = CatalogProduct.new(catalog_products(:smrk).attributes.except("id"))
    assert_not duplicate.valid?
    assert duplicate.errors.of_kind?(:key, :taken)
  end

  test "requires template to be one of the known templates" do
    product = CatalogProduct.new(key: "test", template: "bogus", category: "palivove")
    assert_not product.valid?
    assert product.errors.of_kind?(:template, :inclusion)
  end

  test "hardness is optional but must be hard or soft when present" do
    product = catalog_products(:tramy)
    assert_nil product.hardness
    assert product.valid?

    product.hardness = "bogus"
    assert_not product.valid?
  end

  test "hardness normalizes a blank string (from the admin form's blank option) to nil" do
    product = catalog_products(:tramy)
    product.hardness = ""

    assert product.valid?
    assert_nil product.hardness
  end
end
