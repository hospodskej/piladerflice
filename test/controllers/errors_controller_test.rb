require "test_helper"

class ErrorsControllerTest < ActionDispatch::IntegrationTest
  # The test env shows Rails' debug error pages; switch to what visitors see.
  setup { Rails.application.env_config["action_dispatch.show_detailed_exceptions"] = false }
  teardown { Rails.application.env_config["action_dispatch.show_detailed_exceptions"] = true }

  test "unknown URL renders the branded 404 page in the site layout" do
    get "/this-page-does-not-exist"

    assert_response :not_found
    assert_select "h1", "Tady nic není"
    assert_select "img.not-found-mascot[src*=mascot]"
    assert_select "meta[name=robots][content=noindex]"
    assert_select ".not-found a.btn-primary[href=?]", root_path
    assert_select "header.site-header"
    assert_select "nav.breadcrumb-nav", false
  end

  test "404 page is translated" do
    get "/this-page-does-not-exist", params: { locale: "de" }

    assert_response :not_found
    assert_select "h1", "Hier ist nichts"
  end

  test "missing record renders the same 404 page" do
    get eshop_product_path("does-not-exist")

    assert_response :not_found
    assert_select "h1", "Tady nic není"
  end

  test "non-HTML requests get a bare 404" do
    get "/nothing.json"

    assert_response :not_found
    assert_empty response.body
  end

  test "404 is not indexed or linked in the sitemap" do
    get sitemap_path
    assert_no_match(%r{/404}, response.body)
  end
end
