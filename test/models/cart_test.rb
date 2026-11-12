require "test_helper"

class CartTest < ActiveSupport::TestCase
  setup do
    @session = {}
    @cart = Cart.new(@session)
    @variant = catalog_variants(:tramy_variant)
  end

  test "starts empty" do
    assert @cart.empty?
    assert_equal 0, @cart.total_quantity
  end

  test "add stores price, image, and specs from the variant, not the caller" do
    product = @variant.catalog_product
    product.image.attach(
      io: File.open(Rails.root.join("app/assets/images/eshop/tramy.webp")),
      filename: "tramy.webp"
    )

    line_id = @cart.add(@variant)
    item = @cart.find(line_id)

    assert_equal product.key, item.product_key
    assert_equal Rails.application.routes.url_helpers.rails_blob_path(product.image, only_path: true), item.image
    assert_equal @variant.customer_price_czk, item.unit_price_czk
    assert_equal "rezivo", item.category
    assert_equal @variant.to_cart_specs, item.specs
    assert_equal 1, item.quantity
  end

  test "add stores a nil image when the product has no image attached" do
    line_id = @cart.add(@variant)
    item = @cart.find(line_id)

    assert_nil item.image
  end

  test "add prefers the variant's own image over the product's image" do
    product = @variant.catalog_product
    product.image.attach(
      io: File.open(Rails.root.join("app/assets/images/eshop/tramy.webp")),
      filename: "tramy.webp"
    )
    @variant.image.attach(
      io: File.open(Rails.root.join("app/assets/images/eshop/fosny.webp")),
      filename: "fosny.webp"
    )

    line_id = @cart.add(@variant)
    item = @cart.find(line_id)

    assert_equal Rails.application.routes.url_helpers.rails_blob_path(@variant.image, only_path: true), item.image
  end

  test "adding the same variant twice increments quantity instead of duplicating the line" do
    first_id = @cart.add(@variant)
    second_id = @cart.add(@variant)

    assert_equal first_id, second_id
    assert_equal 1, @cart.items.size
    assert_equal 2, @cart.find(first_id).quantity
  end

  test "update_quantity sets a line's quantity" do
    line_id = @cart.add(@variant)
    @cart.update_quantity(line_id, 5)

    assert_equal 5, @cart.find(line_id).quantity
  end

  test "update_quantity removes the line when quantity drops to zero or below" do
    line_id = @cart.add(@variant)
    @cart.update_quantity(line_id, 0)

    assert_nil @cart.find(line_id)
    assert @cart.empty?
  end

  test "remove deletes a line" do
    line_id = @cart.add(@variant)
    @cart.remove(line_id)

    assert @cart.empty?
  end

  test "subtotal_czk sums quantity times unit price across lines" do
    other = catalog_variants(:odkory_smrk)
    line_id = @cart.add(@variant)
    @cart.update_quantity(line_id, 2)
    @cart.add(other)

    assert_equal (@variant.customer_price_czk * 2) + other.customer_price_czk, @cart.subtotal_czk
  end

  test "lumber entered without VAT costs the customer the price plus 21 %" do
    @cart.add(@variant)

    assert_equal 8_500, @variant.price_czk
    assert_equal 10_285, @cart.subtotal_czk
    assert_equal({ 21 => 1_785 }, @cart.vat_breakdown(country: "cz").by_rate)
  end

  test "to Austria the price stays the same and only the VAT split changes" do
    @cart.add(@variant) # lumber, 10 285 Kč: 20 % in Austria
    @cart.add(catalog_variants(:smrk_kontejner_100cm)) # firewood, entered with VAT: 13 % in Austria

    vat = @cart.vat_breakdown(country: "at")
    firewood = catalog_variants(:smrk_kontejner_100cm).price_czk
    assert_equal 10_285 + firewood, vat.total_czk
    assert_equal({ 20 => VatBreakdown.vat_in(10_285, 20), 13 => VatBreakdown.vat_in(firewood, 13) }, vat.by_rate)
  end

  test "clear empties the cart" do
    @cart.add(@variant)
    @cart.clear

    assert @cart.empty?
  end

  test "a second Cart wrapping the same session sees the same contents" do
    line_id = @cart.add(@variant)

    other_view = Cart.new(@session)
    assert_equal line_id, other_view.find(line_id).id
  end

  test "a discount code lowers the total, not the goods subtotal" do
    @cart.add(@variant, 2)
    DiscountCode.create!(code: "JARO10", kind: "percent", value: 10)

    assert_nil @cart.apply_discount("jaro10")
    assert @cart.discount_applied?
    assert_equal 20_570, @cart.subtotal_czk
    assert_equal 2_057, @cart.discount_czk
    assert_equal 18_513, @cart.total_czk
    assert_equal "JARO10", Cart.new(@session).discount.code
  end

  test "an unknown or unusable code is refused and not kept" do
    @cart.add(@variant)
    DiscountCode.create!(code: "VELKY", kind: "fixed", value: 500, min_order_czk: 11_000)

    assert_equal :not_found, @cart.apply_discount("NIC")
    assert_equal :min_order, @cart.apply_discount("VELKY")
    assert_nil @cart.discount
    assert_equal 10_285, @cart.total_czk
  end

  test "a kept code stops applying when the goods drop under its minimum" do
    line_id = @cart.add(@variant, 2)
    DiscountCode.create!(code: "VELKY", kind: "fixed", value: 500, min_order_czk: 11_000)
    @cart.apply_discount("VELKY")

    @cart.update_quantity(line_id, 1)

    assert_equal :min_order, @cart.discount_unusable_reason
    assert_equal 0, @cart.discount_czk
    assert_equal 10_285, @cart.total_czk
  end

  test "clearing the cart drops the discount code" do
    @cart.add(@variant)
    DiscountCode.create!(code: "JARO10", kind: "percent", value: 10)
    @cart.apply_discount("JARO10")

    @cart.clear

    assert_nil Cart.new(@session).discount
  end
end
