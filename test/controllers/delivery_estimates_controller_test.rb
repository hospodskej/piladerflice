require "test_helper"

class DeliveryEstimatesControllerTest < ActionDispatch::IntegrationTest
  setup do
    fake_route(meters: 23_400, place: "Znojmo")
  end

  test "the e-shop asks where to deliver" do
    get eshop_path

    assert_select "#delivery-estimate form[action='#{delivery_estimate_path}'] input[name=query]"
    assert_select "#delivery-estimate select[name=country] option[selected][value=cz]"
  end

  test "a town gives the approximate delivery price straight away" do
    post delivery_estimate_path, params: { query: "Znojmo", country: "cz" }, as: :turbo_stream

    assert_response :success
    # 23 km there, 46 km there and back, at 40 Kč per km.
    assert_select "#delivery-estimate .delivery-estimate-result", text: /Znojmo.*23 km.*46 km.*1.840 Kč/m
    assert_equal 23, session[:delivery_estimate]["one_way_km"]
  end

  test "the estimate stays for the visit and shows on every page with the note" do
    post delivery_estimate_path, params: { query: "Znojmo", country: "cz" }

    get eshop_product_path(catalog_products(:smrk).key)
    assert_select "#delivery-estimate .delivery-estimate-result", text: /Znojmo/
  end

  test "a new price per km applies to an estimate already made" do
    post delivery_estimate_path, params: { query: "Znojmo", country: "cz" }
    Setting.create!(key: "delivery_czk_per_km", value: "50")

    get eshop_path
    assert_select ".delivery-estimate-result", text: /2.300 Kč/
  end

  test "the cart adds the delivery to the total" do
    post cart_items_path, params: { catalog_variant_id: catalog_variants(:tramy_variant).id }
    post delivery_estimate_path, params: { query: "Znojmo", country: "cz" }, as: :turbo_stream

    assert_select "#cart-page-totals .cart-total-delivery", text: /Znojmo.*46 km.*1.840 Kč/m
    assert_select "#cart-page-totals .cart-total-row strong", text: /≈ 10.340 Kč/
    assert_select "#cart-widget .cart-total-delivery", text: /1.840 Kč/

    get cart_path
    assert_select "#cart-page-totals .cart-total-row strong", text: /≈ 10.340 Kč/
  end

  test "a place that can't be found says so and keeps nothing" do
    fake_route(coordinates: nil)
    post delivery_estimate_path, params: { query: "Neexistuje", country: "cz" }, as: :turbo_stream

    assert_select ".delivery-estimate-error", text: /Neexistuje/
    assert_nil session[:delivery_estimate]
  end

  test "an empty box asks for a place" do
    post delivery_estimate_path, params: { query: " " }, as: :turbo_stream
    assert_select ".delivery-estimate-error", text: "Zadejte obec nebo PSČ."
  end

  test "without Turbo the page comes back with the estimate" do
    post delivery_estimate_path, params: { query: "Znojmo", country: "cz" }, headers: { "HTTP_REFERER" => eshop_url }
    assert_redirected_to eshop_url

    follow_redirect!
    assert_select ".delivery-estimate-result", text: /Znojmo/
  end

  test "the place can be changed again" do
    post delivery_estimate_path, params: { query: "Znojmo", country: "cz" }
    delete delivery_estimate_path, as: :turbo_stream

    assert_nil session[:delivery_estimate]
    assert_select "#delivery-estimate input[name=query]"
  end

  test "the German site estimates in Austria by default and in euros" do
    post delivery_estimate_path(locale: "at"), params: { query: "Retz" }, as: :turbo_stream

    assert_includes @fake_route_urls.first, "boundary.country=AT"
    assert_select ".delivery-estimate-result", text: /€/
  end

  test "checkout starts from the place: the estimate at the shipping step, the town and PSČ in the details" do
    post cart_items_path, params: { catalog_variant_id: catalog_variants(:tramy_variant).id }
    post delivery_estimate_path, params: { query: "Znojmo", country: "cz" }

    get checkout_shipping_path
    assert_select ".checkout-radio-note", text: /Znojmo: ≈ 1.840 Kč/

    patch checkout_shipping_path, params: { shipping_method: "address", payment_method: "cash", country: "cz" }
    get checkout_details_path
    assert_select "input#billing_city[value=Znojmo]"
    assert_select "input#billing_zip[value='669 02']"
  end
end
