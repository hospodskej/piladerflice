require "test_helper"

class CartHelperTest < ActionView::TestCase
  test "a line describes the product by its title" do
    item = CartLineItem.new("0", "product_key" => "smrk", "specs" => [])

    assert_equal catalog_products(:smrk).title, cart_line_description(item)
  end

  test "a product no longer sold still gets a readable name" do
    item = CartLineItem.new("0", "product_key" => "zmizely_drevo", "specs" => [])

    assert_equal "Zmizely drevo", cart_line_description(item)
  end
end
