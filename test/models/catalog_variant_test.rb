require "test_helper"

class CatalogVariantTest < ActiveSupport::TestCase
  test "to_cart_specs for a loose (kontejner) firewood variant" do
    variant = catalog_variants(:smrk_kontejner_100cm)

    assert_equal [
      { "type" => "key", "value" => "common.loose_variant" },
      { "type" => "raw", "value" => "100 cm" },
      { "type" => "amount", "value" => 15, "unit_key" => "common.per_m3" }
    ], variant.to_cart_specs
  end

  test "to_cart_specs for a stacked (bedny) firewood variant" do
    variant = catalog_variants(:smrk_bedny)

    assert_equal "common.stacked_variant", variant.to_cart_specs.first["value"]
  end

  test "to_cart_specs for a lumber variant" do
    variant = catalog_variants(:tramy_variant)

    assert_equal [
      { "type" => "raw", "value" => "3 000 mm" },
      { "type" => "raw", "value" => "100×100 mm" }
    ], variant.to_cart_specs
  end

  test "to_cart_specs for a lumber variant with a grade" do
    variant = catalog_variants(:tramy_variant)
    variant.grade = "I. tříděné"
    variant.grade_de = "I. sortiert"

    assert_equal [
      { "type" => "bilingual", "value" => "I. tříděné", "value_de" => "I. sortiert" },
      { "type" => "raw", "value" => "3 000 mm" },
      { "type" => "raw", "value" => "100×100 mm" }
    ], variant.to_cart_specs
  end

  test "to_cart_specs for a simple_variant variant" do
    variant = catalog_variants(:odkory_smrk)

    assert_equal [
      { "type" => "bilingual", "value" => "Smrk", "value_de" => "Fichte" }
    ], variant.to_cart_specs
  end

  test "requires a key unique within its product, but not across products" do
    tramy = catalog_products(:tramy)
    duplicate = tramy.catalog_variants.new(catalog_variants(:tramy_variant).attributes.except("id", "catalog_product_id"))
    assert_not duplicate.valid?
    assert duplicate.errors.of_kind?(:key, :taken)

    elsewhere = catalog_products(:smrk).catalog_variants.new(
      key: catalog_variants(:tramy_variant).key, price_czk: 100
    )
    assert elsewhere.valid?
  end

  test "requires a positive price" do
    variant = catalog_variants(:tramy_variant)
    variant.price_czk = 0
    assert_not variant.valid?

    variant.price_czk = -5
    assert_not variant.valid?
  end

  test "variant_group is optional but must be kontejner or bedny when present" do
    variant = catalog_variants(:tramy_variant)
    assert_nil variant.variant_group
    assert variant.valid?

    variant.variant_group = "bogus"
    assert_not variant.valid?
  end
end
