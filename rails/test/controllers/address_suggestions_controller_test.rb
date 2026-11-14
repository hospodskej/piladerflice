require "test_helper"

class AddressSuggestionsControllerTest < ActionDispatch::IntegrationTest
  test "returns suggestions as JSON, starting no session" do
    fake_route do |_url|
      { "features" => [{ "properties" => { "label" => "Brno, Czechia", "locality" => "Brno", "postalcode" => "602 00" } }] }
    end

    get address_suggestions_path, params: { q: "Brn", kind: "place", country: "cz" }

    assert_response :success
    assert_equal [{ "label" => "Brno, Czechia", "street" => nil, "city" => "Brno", "zip" => "602 00", "country" => "cz" }], response.parsed_body
    assert_nil response.headers["Set-Cookie"]
  end

  test "the town box and both checkout street fields suggest as you type" do
    get eshop_path
    assert_select "form.delivery-estimate-form[data-controller=address-autocomplete][data-address-autocomplete-kind-value=place] input#delivery-estimate-query[data-address-autocomplete-target=input]"

    post cart_items_path, params: { catalog_variant_id: catalog_variants(:tramy_variant).id }
    patch checkout_shipping_path, params: { shipping_method: "address", payment_method: "cash" }
    get checkout_details_path
    assert_select "[data-controller=address-autocomplete][data-address-autocomplete-city-value='#billing_city'] input#billing_street[data-address-autocomplete-target=input]"
    assert_select "[data-controller=address-autocomplete][data-address-autocomplete-city-value='#delivery_city'] input#delivery_street[data-address-autocomplete-target=input]"
  end
end
