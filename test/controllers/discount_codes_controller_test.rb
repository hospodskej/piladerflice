require "test_helper"

class DiscountCodesControllerTest < ActionDispatch::IntegrationTest
  setup do
    post cart_items_path, params: { catalog_variant_id: catalog_variants(:tramy_variant).id }
    DiscountCode.create!(code: "JARO10", kind: "percent", value: 10)
  end

  test "the cart has a working discount code box" do
    get cart_path
    assert_select "form[action='#{cart_discount_path}'] input[name=code]:not([disabled])"
  end

  test "a valid code shows the discount in the cart and the dropdown" do
    post cart_discount_path, params: { code: "jaro10" }, as: :turbo_stream

    assert_response :success
    assert_select ".discount-code-chip", text: "JARO10 (−10 %)"
    assert_select ".discount-code-message", text: "Slevový kód JARO10 byl uplatněn."
    assert_select "#cart-widget .cart-total-discount", text: /850/
    assert_select "#cart-page-contents .cart-total-row strong", text: /7.650/
  end

  test "an unknown code shows why without keeping it" do
    post cart_discount_path, params: { code: "NIC" }, as: :turbo_stream

    assert_select ".discount-code-message-error", text: "Tento slevový kód neexistuje."
    assert_nil session[:discount_code]
  end

  test "a code under its minimum says from what amount it applies" do
    DiscountCode.create!(code: "VELKY", kind: "fixed", value: 500, min_order_czk: 10_000)
    post cart_discount_path, params: { code: "VELKY" }, as: :turbo_stream

    assert_select ".discount-code-message-error", text: /od 10.000 Kč/
  end

  test "without Turbo the cart page shows the result after a redirect" do
    post cart_discount_path, params: { code: "NIC" }
    assert_redirected_to cart_path

    follow_redirect!
    assert_select ".discount-code-message-error", text: "Tento slevový kód neexistuje."
  end

  test "the code can be removed again" do
    post cart_discount_path, params: { code: "JARO10" }
    delete cart_discount_path, as: :turbo_stream

    assert_nil session[:discount_code]
    assert_select ".cart-total-discount", count: 0
  end
end
