require "test_helper"

# IDOR: tries to reach or change other people's data by putting their IDs
# into the request, as a guest and as a logged-in customer.
class IdorTest < ActionDispatch::IntegrationTest
  def sign_in_as(email)
    User.create!(email_address: email, password: "supersecret", role: "customer") unless User.exists?(email_address: email)
    post session_path, params: { email_address: email, password: "supersecret" }
  end

  # Every admin route, with real IDs filled in.
  def admin_requests
    product = catalog_products(:tramy)
    ids = {
      id: [orders(:this_month), Inquiry.create!(first_name: "A", last_name: "B", email: "a@b.cz", phone: "1",
                                                items_snapshot: { "palivove" => [{ "druh" => "Dub" }] }.to_json)].first.id,
      catalog_product_id: product.id
    }
    Rails.application.routes.routes.filter_map do |route|
      path = route.path.spec.to_s.sub("(.:format)", "")
      next unless path.start_with?("/admin")

      verb = route.verb.presence || "GET"
      filled = path.gsub(/:(\w+)/) { (ids[$1.to_sym] || 1).to_s }
      [verb.split("|").first.downcase.to_sym, filled]
    end.uniq
  end

  def assert_admin_closed(who)
    requests = admin_requests
    counts = [Order.count, Inquiry.count, CatalogProduct.count, CatalogVariant.count, DiscountCode.count, Promo.count, PricelistItem.count, InquiryFormOption.count, Setting.count]
    assert requests.size > 30, "expected to find the admin routes"

    requests.each do |verb, path|
      send(verb, path, params: { setting: { value: "999" }, discount_code: { code: "HACK", kind: "fixed", value: "1" } })
      assert response.redirect?, "#{who}: #{verb.upcase} #{path} answered #{response.status}, expected a redirect away"
      refute_match %r{/admin}, response.location.to_s, "#{who}: #{verb.upcase} #{path} redirected inside the admin"
    end
    assert_equal counts, [Order.count, Inquiry.count, CatalogProduct.count, CatalogVariant.count, DiscountCode.count, Promo.count, PricelistItem.count, InquiryFormOption.count, Setting.count], "#{who} changed data"
  end

  test "a guest can't reach any admin page or action" do
    assert_admin_closed("guest")
  end

  test "a logged-in customer can't reach any admin page or action" do
    sign_in_as("zakaznik@example.com")
    assert_admin_closed("customer")
  end

  test "a customer can't reorder someone else's order" do
    victim = User.create!(email_address: "obet@example.com", password: "supersecret", role: "customer")
    order = orders(:this_month)
    order.update!(user: victim, items_snapshot: [{ catalog_variant_id: catalog_variants(:tramy_variant).id, quantity: 3 }].to_json)

    sign_in_as("utocnik@example.com")
    post reorder_order_path(order)

    assert_response :not_found
    assert_empty session[:cart].to_h
  end

  test "a customer's account lists only their own orders" do
    victim = User.create!(email_address: "obet2@example.com", password: "supersecret", role: "customer")
    orders(:this_month).update!(user: victim)

    sign_in_as("utocnik2@example.com")
    get account_path
    assert_select ".order-card", count: 0
  end

  test "the order confirmation page shows only the order placed in this browser" do
    get checkout_confirmation_path
    assert_redirected_to cart_path

    get checkout_confirmation_path, params: { id: orders(:this_month).id, order_id: orders(:this_month).id }
    assert_redirected_to cart_path
  end

  test "cart lines are this browser's only: another line ID changes nothing" do
    post cart_items_path, params: { catalog_variant_id: catalog_variants(:tramy_variant).id }
    before = session[:cart].deep_dup

    patch cart_item_path(999_999), params: { quantity: 50 }
    delete cart_item_path(888_888)
    assert_equal before, session[:cart]
  end

  test "an inactive product can't be put in the cart by its variant ID" do
    post cart_items_path, params: { catalog_variant_id: catalog_variants(:inactive_variant).id }

    assert_response :not_found
    assert_empty session[:cart].to_h
  end

  test "the profile picture address has no ID: everyone gets only their own" do
    sign_in_as("bez-obrazku@example.com")
    get avatar_path(id: 1, user_id: 1)
    assert_response :not_found
  end

  test "a forged session cookie doesn't log anyone in" do
    cookies[:session_id] = Session.first&.id.to_s.presence || "1"
    get account_path
    assert_redirected_to login_path
  end
end
