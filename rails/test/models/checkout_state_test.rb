require "test_helper"

class CheckoutStateTest < ActiveSupport::TestCase
  setup do
    @state = CheckoutState.new({})
  end

  test "update assigns permitted string attributes" do
    @state.update(first_name: "Jana", not_a_real_attribute: "ignored")

    assert_equal "Jana", @state["first_name"]
    assert_nil @state["not_a_real_attribute"]
  end

  test "update casts boolean attributes from checkbox-style values" do
    @state.update(newsletter_opt_in: "1")
    assert @state.newsletter_opt_in?

    @state.update(newsletter_opt_in: "0")
    assert_not @state.newsletter_opt_in?
  end

  test "boolean attributes read as falsy before anything is submitted" do
    assert_not @state.newsletter_opt_in?
    assert_not @state.company_purchase?
    assert_not @state.delivery_address_different?
  end

  test "shipping_selected? requires both a shipping and payment method" do
    assert_not @state.shipping_selected?

    @state.update(shipping_method: "pickup")
    assert_not @state.shipping_selected?

    @state.update(payment_method: "cash")
    assert @state.shipping_selected?
  end

  test "details_complete? requires every contact and billing field" do
    assert_not @state.details_complete?

    @state.update(
      first_name: "Jana", last_name: "Nováková", email: "jana@example.com", phone: "123456789",
      billing_street: "Hlavní 1", billing_city: "Brno", billing_zip: "60200", billing_country: "cz"
    )
    assert @state.details_complete?
  end

  test "clear resets all stored attributes" do
    @state.update(first_name: "Jana", newsletter_opt_in: "1")
    @state.clear

    assert_nil @state["first_name"]
    assert_not @state.newsletter_opt_in?
  end
end
