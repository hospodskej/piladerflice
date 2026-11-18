require "test_helper"

class PricelistEshopStockTest < ActiveSupport::TestCase
  def row(category, item_name, subcategory: nil)
    PricelistItem.new(category: category, subcategory: subcategory, item_name: item_name, price: "1 Kč", available: true)
  end

  test "a firewood row follows the variants of its wood, kind and length" do
    stock = PricelistEshopStock.new
    assert_equal true, stock.available?(row("palivove_volne", "1m", subcategory: "Smrk / Borovice / Modřín"))
    assert_equal false, stock.available?(row("palivove_volne", "50cm", subcategory: "Smrk / Borovice / Modřín"))
    assert_equal true, stock.available?(row("palivove_skladane", "50cm", subcategory: "Smrk"))
    assert_equal [ "Smrk" ], stock.product_titles(row("palivove_volne", "1m", subcategory: "Smrk / Borovice / Modřín"))
  end

  test "a lumber or by-product row follows the product of its name" do
    stock = PricelistEshopStock.new
    assert_equal true, stock.available?(row("stavebni", "Trámy"))
    assert_equal true, stock.available?(row("zbytky", "Odkory na topení"))

    catalog_variants(:tramy_variant).update!(in_stock: false)
    assert_equal false, PricelistEshopStock.new.available?(row("stavebni", "Trámy"))
  end

  test "a row the e-shop doesn't sell has no answer, so its checkbox applies" do
    stock = PricelistEshopStock.new
    assert_nil stock.available?(row("palivove_volne", "1m", subcategory: "Ostatní tvrdé"))
    assert_nil stock.available?(row("palivove_volne", "25cm", subcategory: "Smrk"))
    assert_nil stock.available?(row("stavebni", "Fošny"))
    assert_nil stock.available?(row("sluzby", "Doprava"))

    item = row("stavebni", "Fošny")
    item.available = false
    assert_not item.available_now?
  end

  test "lengths are compared in centimetres and names without accents or notes" do
    assert_equal 100, PricelistEshopStock.length_cm("1m")
    assert_equal 100, PricelistEshopStock.length_cm("100 cm")
    assert_equal 33, PricelistEshopStock.length_cm("33cm")
    assert_nil PricelistEshopStock.length_cm("Odkory")
    assert_equal "prkna", PricelistEshopStock.normalize("Prkna (24mm)")
    assert_equal "stresni late", PricelistEshopStock.normalize("Střešní latě")
  end
end
