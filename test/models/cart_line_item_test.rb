require "test_helper"

class CartLineItemTest < ActiveSupport::TestCase
  test "line_total_czk multiplies unit price by quantity" do
    item = CartLineItem.new("1", { "unit_price_czk" => 150, "quantity" => 3 })
    assert_equal 450, item.line_total_czk
  end

  test "casts unit_price_czk and quantity from stored strings" do
    item = CartLineItem.new("1", { "unit_price_czk" => "150", "quantity" => "3" })
    assert_equal 150, item.unit_price_czk
    assert_equal 3, item.quantity
  end

  test "defaults specs to an empty array when missing" do
    item = CartLineItem.new("1", {})
    assert_equal [], item.specs
  end
end
