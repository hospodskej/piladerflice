require "test_helper"

class CartItemsControllerTest < ActionDispatch::IntegrationTest
  setup do
    @variant = catalog_variants(:tramy_variant)
  end

  test "create adds the variant to the cart" do
    post cart_items_path, params: { catalog_variant_id: @variant.id }

    assert_redirected_to root_path
    assert_equal 1, session[:cart].size
    added = session[:cart].values.first
    assert_equal @variant.catalog_product.key, added["product_key"]
    assert_equal @variant.price_czk, added["unit_price_czk"]
  end

  test "create 404s for a variant belonging to an inactive product" do
    post cart_items_path, params: { catalog_variant_id: catalog_variants(:inactive_variant).id }
    assert_response :not_found
  end

  test "create 404s for an unknown variant id" do
    post cart_items_path, params: { catalog_variant_id: 0 }
    assert_response :not_found
  end

  test "update sets an existing line's quantity" do
    post cart_items_path, params: { catalog_variant_id: @variant.id }
    line_id = session[:cart].keys.first

    patch cart_item_path(line_id), params: { quantity: "5" }

    assert_redirected_to cart_path
    assert_equal 5, session[:cart][line_id]["quantity"]
  end

  test "update removes the line when quantity drops to zero" do
    post cart_items_path, params: { catalog_variant_id: @variant.id }
    line_id = session[:cart].keys.first

    patch cart_item_path(line_id), params: { quantity: "0" }

    assert_redirected_to cart_path
    assert_not session[:cart].key?(line_id)
  end

  test "destroy removes a line from the cart" do
    post cart_items_path, params: { catalog_variant_id: @variant.id }
    line_id = session[:cart].keys.first

    delete cart_item_path(line_id)

    assert_redirected_to cart_path
    assert_not session[:cart].key?(line_id)
  end
end
