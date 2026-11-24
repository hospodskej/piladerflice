require "test_helper"

# The Austrian site (/at) shows € prices Štěpán sets in the admin, and an
# order placed there is in €. Without a € price it falls back to the day's
# exchange rate ("≈").
class EuroPricesTest < ActionDispatch::IntegrationTest
  setup do
    Rails.cache.write(ExchangeRateService::CACHE_KEY, 25.0, expires_in: ExchangeRateService::CACHE_TTL)
    @lumber = catalog_variants(:tramy_variant)       # lumber: entered without VAT (+21 %)
    @firewood = catalog_variants(:smrk_kontejner_100cm) # firewood: entered with VAT
  end

  test "a € price from the admin is shown exactly; without one the Kč price is converted" do
    @lumber.update!(price_eur: 100)
    get "/at/eshop/#{@lumber.catalog_product.key}"
    assert_select ".inventory-price", text: /\A121,00 € /  # 100 € + 21 % like the Kč price

    get "/at/eshop/#{@firewood.catalog_product.key}"
    assert_select ".inventory-price", text: /≈ 60,00 €/      # 1 500 Kč / 25

    get "/eshop/#{@lumber.catalog_product.key}"
    assert_select ".inventory-price", text: /10 285 Kč/      # the Czech site is unchanged
  end

  test "the Austrian cart adds up the € prices" do
    @lumber.update!(price_eur: 100)
    @firewood.update!(price_eur: 59.90)
    post "/at/cart_items", params: { catalog_variant_id: @lumber.id }
    post "/at/cart_items", params: { catalog_variant_id: @firewood.id, quantity: 2 }

    get "/at/kosik"
    assert_select "#cart-page-totals .cart-total-row strong", text: "240,80 €" # 121 + 2 × 59,90
  end

  test "an order placed on the Austrian site is in € everywhere" do
    @lumber.update!(price_eur: 100)
    Setting.create!(key: "delivery_eur_per_km", value: "1.5")
    post "/at/cart_items", params: { catalog_variant_id: @lumber.id }
    patch "/at/kosik/doprava", params: { shipping_method: "pickup", payment_method: "bank_transfer" }
    patch "/at/kosik/udaje", params: {
      first_name: "Anna", last_name: "Huber", email: "anna@example.at", phone: "+43 123",
      billing_street: "Hauptplatz 1", billing_city: "Retz", billing_zip: "2070", billing_country: "at",
      company_purchase: "0", delivery_address_different: "0", newsletter_opt_in: "0"
    }

    get "/at/kosik/souhrn"
    assert_select ".cart-total-row strong", text: "121,00 €"

    assert_emails 2 do
      post "/at/kosik/souhrn", params: { terms_agreement: "1" }
    end
    order = Order.last
    assert order.eur?
    assert_equal "121.00", order.eur_amounts["total"]
    assert_equal BigDecimal("21.00"), order.vat_by_rate_in_currency[21] # pickup in Derflice: Czech VAT
    assert_equal 10_285, order.total_czk # the Kč price is kept too

    shop_mail, customer_mail = ActionMailer::Base.deliveries.last(2)
    assert_includes customer_mail.text_part.body.to_s, "121,00 €"
    assert_includes shop_mail.text_part.body.to_s, "CELKEM K ÚHRADĚ (včetně DPH): 121,00 €"
  end

  test "a fixed discount uses its € value on the Austrian site" do
    @lumber.update!(price_eur: 100)
    DiscountCode.create!(code: "WIEN", kind: "fixed", value: 500, value_eur: 20)
    post "/at/cart_items", params: { catalog_variant_id: @lumber.id }
    post "/at/kosik/slevovy-kod", params: { code: "WIEN" }

    get "/at/kosik"
    assert_select ".discount-code-chip", text: "WIEN (−20,00 €)"
    assert_select "#cart-page-totals .cart-total-row strong", text: "101,00 €"
  end

  test "Czech orders stay in Kč" do
    post cart_items_path, params: { catalog_variant_id: @firewood.id }
    patch checkout_shipping_path, params: { shipping_method: "pickup", payment_method: "cash" }
    patch checkout_details_path, params: {
      first_name: "Jana", last_name: "Nováková", email: "jana@example.com", phone: "123456789",
      billing_street: "Hlavní 1", billing_city: "Brno", billing_zip: "60200", billing_country: "cz",
      company_purchase: "0", delivery_address_different: "0", newsletter_opt_in: "0"
    }
    post checkout_confirm_path, params: { terms_agreement: "1" }

    assert_not Order.last.eur?
    assert_equal "1 500 Kč", Order.last.display(:total)
  end

  test "the delivery € price per km from the admin is what the Austrian site quotes" do
    Setting.create!(key: "delivery_eur_per_km", value: "1.5")
    get "/at/kontakt"
    assert_select ".cenik-sluzba-box", text: "1,50 € / km"
    get "/kontakt"
    assert_select ".cenik-sluzba-box", text: "40 Kč / km"
  end

  test "the admin saves the € prices" do
    post session_path, params: { email_address: users(:admin).email_address, password: "correct-horse-battery" }

    patch admin_catalog_product_catalog_variant_path(@lumber.catalog_product, @lumber), params: { catalog_variant: { price_eur: "99.50" } }
    assert_equal BigDecimal("99.50"), @lumber.reload.price_eur

    patch admin_delivery_settings_path, params: { setting: { value: "40", value_eur: "1,6" } }
    assert_equal BigDecimal("1.6"), Setting.delivery_eur_per_km

    patch admin_delivery_settings_path, params: { setting: { value: "40", value_eur: "" } }
    assert_nil Setting.delivery_eur_per_km

    get edit_admin_catalog_product_catalog_variant_path(@lumber.catalog_product, @lumber)
    assert_select "label[for=catalog_variant_price_eur]", text: /Cena \(€, bez DPH\)/
  end
end
