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

  test "the address delivery option shows the price per km" do
    get checkout_shipping_path
    assert_select ".checkout-radio-note", text: "40 Kč/km – orientační cenu uvidíte v souhrnu objednávky"
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
    assert_select "img.checkout-confirmation-mascot[src*=thumbs-up]"
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

  test "a guest checkout creates an order with no linked user" do
    submit_shipping
    submit_details
    post checkout_confirm_path, params: { terms_agreement: "1" }

    assert_nil Order.last.user_id
  end

  test "a logged-in customer's order is linked to their account" do
    customer = User.create!(email_address: "zakaznik@example.com", password: "supersecret123", role: "customer")
    post session_path, params: { email_address: "zakaznik@example.com", password: "supersecret123" }

    submit_shipping
    submit_details
    post checkout_confirm_path, params: { terms_agreement: "1" }

    assert_equal customer, Order.last.user
  end

  test "checkout details are saved to the account after a successful order" do
    customer = User.create!(email_address: "zakaznik@example.com", password: "supersecret123", role: "customer")
    post session_path, params: { email_address: "zakaznik@example.com", password: "supersecret123" }

    submit_shipping
    submit_details(first_name: "Petr", last_name: "Svoboda", billing_city: "Praha")
    post checkout_confirm_path, params: { terms_agreement: "1" }

    customer.reload
    assert_equal "Petr", customer.first_name
    assert_equal "Svoboda", customer.last_name
    assert_equal "Praha", customer.billing_city
  end

  test "the details page prefills from a previously saved account profile" do
    User.create!(
      email_address: "zakaznik@example.com", password: "supersecret123", role: "customer",
      first_name: "Eva", last_name: "Dvořáková", phone: "111222333",
      billing_street: "Krátká 3", billing_city: "Znojmo", billing_zip: "67128", billing_country: "cz"
    )
    post session_path, params: { email_address: "zakaznik@example.com", password: "supersecret123" }

    submit_shipping
    get checkout_details_path

    assert_match 'value="Eva"', response.body
    assert_match 'value="Znojmo"', response.body
  end

  test "the account prefill does not overwrite details already entered this session" do
    User.create!(
      email_address: "zakaznik@example.com", password: "supersecret123", role: "customer",
      first_name: "Eva"
    )
    post session_path, params: { email_address: "zakaznik@example.com", password: "supersecret123" }

    submit_shipping
    submit_details(first_name: "Jana")
    get checkout_details_path

    assert_match 'value="Jana"', response.body
    assert_no_match 'value="Eva"', response.body
  end

  test "delivery to an address is priced at 40 Kč per km there and back and added to the total" do
    fake_route(meters: 50_000)
    submit_shipping(shipping_method: "address", payment_method: "bank_transfer")
    submit_details

    get checkout_summary_path
    assert_response :success
    assert_select ".checkout-radio-note", text: /4 000 Kč\s*\(orientační cena\)/
    assert_select ".cart-total-row", text: /Doprava \(100 km, orientačně\)\s*4 000 Kč/
    assert_select ".cart-total-row strong", text: ActiveSupport::NumberHelper.number_to_delimited(@variant.price_czk + 4000, delimiter: " ") + " Kč"

    assert_emails 2 do
      post checkout_confirm_path, params: { terms_agreement: "1" }
    end

    order = Order.last
    assert_equal 100, order.delivery_distance_km
    assert_equal 4000, order.delivery_czk
    assert_equal @variant.price_czk, order.subtotal_czk
    assert_equal @variant.price_czk + 4000, order.total_czk
    assert_equal order.total_czk - (order.total_czk / 1.21).round, order.vat_czk
    ActionMailer::Base.deliveries.last(2).each do |mail|
      assert_includes mail.text_part.body.to_s, "Doprava (100 km tam i zpět, orientačně): 4 000 Kč"
    end
  end

  test "a different delivery address is the one the distance is measured to" do
    fake_route
    submit_shipping(shipping_method: "address", payment_method: "bank_transfer")
    submit_details(delivery_address_different: "1", delivery_street: "Hauptplatz 1", delivery_city: "Retz",
                   delivery_zip: "2070", delivery_country: "at")

    get checkout_summary_path
    assert_includes @fake_route_urls.first, "locality=Retz"
    assert_includes @fake_route_urls.first, "country=AT"
  end

  test "without a distance the delivery is priced individually and left out of the total" do
    fake_route(coordinates: nil)
    submit_shipping(shipping_method: "address", payment_method: "bank_transfer")
    submit_details

    get checkout_summary_path
    assert_response :success
    assert_includes response.body, "Ceny dopravy jsou kalkulovány individuálně"
    assert_select ".cart-total-row", text: /Doprava/, count: 0

    post checkout_confirm_path, params: { terms_agreement: "1" }
    order = Order.last
    assert_nil order.delivery_czk
    assert_equal @variant.price_czk, order.total_czk
    assert_includes ActionMailer::Base.deliveries.last(2).first.text_part.body.to_s, "Vzdálenost se nepodařilo spočítat"
  end

  test "pickup has no delivery price and does not look up a distance" do
    fake_route
    submit_shipping
    submit_details

    get checkout_summary_path
    assert_select ".cart-total-row", text: /Doprava/, count: 0
    assert_empty @fake_route_urls
  end

  test "the Austrian checkout shows the delivery price in euros" do
    Rails.cache.write(ExchangeRateService::CACHE_KEY, 25.0, expires_in: ExchangeRateService::CACHE_TTL)
    fake_route(meters: 50_000)
    submit_shipping(shipping_method: "address", payment_method: "bank_transfer")
    submit_details(billing_street: "Hauptplatz 1", billing_city: "Retz", billing_zip: "2070", billing_country: "at")

    get "/at/kosik/souhrn"
    assert_response :success
    assert_select ".checkout-radio-note", text: /≈ 160,00 €\s*\(ungefährer Preis\)/
    assert_select ".cart-total-row", text: /Lieferung \(100 km, ungefähr\)\s*≈ 160,00 €/
  end
end
