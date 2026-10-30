require "test_helper"

class LlmsControllerTest < ActionDispatch::IntegrationTest
  test "llms.txt describes the business in Markdown with the current catalogue and delivery price" do
    Setting.create!(key: "delivery_czk_per_km", value: "45")

    get "/llms.txt"

    assert_response :success
    assert_equal "text/plain", response.media_type
    body = response.body
    assert body.start_with?("# Pila Derflice\n\n> ")
    assert_includes body, "45 Kč per km"
    assert_includes body, "stepan.merta@seznam.cz"
    assert_includes body, "[#{catalog_products(:tramy).title}](http://www.example.com/eshop/tramy)"
    assert_includes body, "from 8 500 Kč per m³"
    assert_includes body, "(http://www.example.com/at/kontakt)"
    assert_not_includes body, catalog_products(:inactive_product).title
  end

  test "it's cacheable and starts no session" do
    get "/llms.txt"

    assert_includes response.headers["Cache-Control"], "public"
    assert_nil response.headers["Set-Cookie"]
  end
end
