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
      io: File.open(Rails.root.join("app/assets/images/eshop/tramy.png")),
      filename: "tramy.png"
    )

    line_id = @cart.add(@variant)
    item = @cart.find(line_id)

    assert_equal product.key, item.product_key
    assert_equal Rails.application.routes.url_helpers.rails_blob_path(product.image, only_path: true), item.image
    assert_equal @variant.price_czk, item.unit_price_czk
    assert_equal @variant.to_cart_specs, item.specs
    assert_equal 1, item.quantity
  end

  test "add stores a nil image when the product has no image attached" do
    line_id = @cart.add(@variant)
    item = @cart.find(line_id)

    assert_nil item.image
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

    assert_equal (@variant.price_czk * 2) + other.price_czk, @cart.subtotal_czk
  end

  test "subtotal_excl_vat_czk backs the 21% VAT rate out of the total" do
    @cart.add(@variant)

    expected = (@cart.subtotal_czk / (1 + Cart::VAT_RATE)).round
    assert_equal expected, @cart.subtotal_excl_vat_czk
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
end
