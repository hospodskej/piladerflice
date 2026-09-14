require "test_helper"

class CheckoutControllerTest < ActionDispatch::IntegrationTest
  setup do
    @variant = catalog_variants(:tramy_variant)
    post cart_items_path, params: { catalog_variant_id: @variant.id }
  end

  def submit_shipping(shipping_method: "pickup", payment_method: "cash")
    patch checkout_shipping_path, params: { shipping_method: shipping_method, payment_method: payment_method }
  end

  def submit_details(overrides = {})
    patch checkout_details_path, params: {
      first_name: "Jana", last_name: "Nováková", email: "jana@example.com", phone: "123456789",
      newsletter_opt_in: "0",
      billing_street: "Hlavní 1", billing_city: "Brno", billing_zip: "60200", billing_country: "cz",
      company_purchase: "0",
      delivery_address_different: "0"
    }.merge(overrides)
  end

  test "shipping redirects to the cart when the cart is empty" do
    delete cart_item_path(session[:cart].keys.first)

    get checkout_shipping_path
    assert_redirected_to cart_path
  end

  test "shipping renders when the cart has items" do
    get checkout_shipping_path
    assert_response :success
  end

  test "details redirects back to shipping until a shipping method is chosen" do
    get checkout_details_path
    assert_redirected_to checkout_shipping_path
  end

  test "summary redirects back to details until contact details are complete" do
    submit_shipping
    get checkout_summary_path
    assert_redirected_to checkout_details_path
  end

  test "the full checkout flow creates an order, emails, and clears the cart" do
    submit_shipping
    assert_redirected_to checkout_details_path

    submit_details
    assert_redirected_to checkout_summary_path

    get checkout_summary_path
    assert_response :success

    assert_emails 2 do
      assert_difference -> { Order.count }, 1 do
        post checkout_confirm_path, params: { terms_agreement: "1" }
      end
    end
    assert_redirected_to checkout_confirmation_path

    order = Order.last
    assert_equal "Jana", order.first_name
    assert_equal @variant.price_czk, order.subtotal_czk
    assert_equal [ { "product_key" => @variant.catalog_product.key } ], order.items.map { |i| i.slice("product_key") }
    assert session[:cart].empty?

    get checkout_confirmation_path
    assert_response :success
  end

  test "confirm re-renders the summary with an error when terms are not agreed" do
    submit_shipping
    submit_details

    assert_no_difference -> { Order.count } do
      post checkout_confirm_path, params: {}
    end
    assert_response :unprocessable_entity
  end

  test "confirm re-renders the summary when the order itself fails validation (company_purchase without a company_name)" do
    submit_shipping
    submit_details(company_purchase: "1")

    assert_no_difference -> { Order.count } do
      post checkout_confirm_path, params: { terms_agreement: "1" }
    end
    assert_response :unprocessable_entity
  end

  test "confirmation redirects to the cart when there is no last order in the session" do
    get checkout_confirmation_path
    assert_redirected_to cart_path
  end
end
