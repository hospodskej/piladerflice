require "test_helper"

class OrderTest < ActiveSupport::TestCase
  def valid_attributes
    {
      first_name: "Jana", last_name: "Nováková", email: "jana@example.com", phone: "123456789",
      billing_street: "Hlavní 1", billing_city: "Brno", billing_zip: "60200", billing_country: "cz",
      shipping_method: "pickup", payment_method: "cash",
      items_snapshot: "[]", subtotal_czk: 1000, vat_czk: 174, total_czk: 1000, locale: "cs"
    }
  end

  test "valid with all required attributes" do
    assert Order.new(valid_attributes).valid?
  end

  test "requires contact and billing details" do
    order = Order.new(valid_attributes.except(:email, :billing_city))
    assert_not order.valid?
    assert order.errors.of_kind?(:email, :blank)
    assert order.errors.of_kind?(:billing_city, :blank)
  end

  test "requires a known shipping and payment method" do
    order = Order.new(valid_attributes.merge(shipping_method: "teleport", payment_method: "crypto"))
    assert_not order.valid?
    assert order.errors.of_kind?(:shipping_method, :inclusion)
    assert order.errors.of_kind?(:payment_method, :inclusion)
  end

  test "requires company_name when company_purchase is set" do
    order = Order.new(valid_attributes.merge(company_purchase: true))
    assert_not order.valid?
    assert order.errors.of_kind?(:company_name, :blank)

    order.company_name = "Dřevo s.r.o."
    assert order.valid?
  end

  test "requires a delivery address when delivery_address_different is set" do
    order = Order.new(valid_attributes.merge(delivery_address_different: true))
    assert_not order.valid?
    assert order.errors.of_kind?(:delivery_street, :blank)
  end

  test "items parses the JSON snapshot" do
    order = Order.new(valid_attributes.merge(items_snapshot: [ { "product_key" => "smrk" } ].to_json))
    assert_equal [ { "product_key" => "smrk" } ], order.items
  end

  test "full_name joins first and last name" do
    order = Order.new(valid_attributes)
    assert_equal "Jana Nováková", order.full_name
  end

  test "billing_country_name resolves the code for the given locale" do
    order = Order.new(valid_attributes.merge(billing_country: "at"))
    assert_equal "Rakousko", order.billing_country_name(locale: :cs)
    assert_equal "Österreich", order.billing_country_name(locale: :de)
  end

  test "billing_country_name returns the raw code when unknown" do
    order = Order.new(valid_attributes.merge(billing_country: "xx"))
    assert_equal "xx", order.billing_country_name(locale: :cs)
  end
end
