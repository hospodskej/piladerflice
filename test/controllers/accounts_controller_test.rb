require "test_helper"

class AccountsControllerTest < ActionDispatch::IntegrationTest
  test "show redirects to login when not authenticated" do
    get account_path
    assert_redirected_to login_path
  end

  test "show renders for a logged-in customer without an admin panel link" do
    User.create!(email_address: "zakaznik@example.com", password: "supersecret", role: "customer")
    post session_path, params: { email_address: "zakaznik@example.com", password: "supersecret" }

    get account_path
    assert_response :success
    assert_no_match "Přejít do administrace", response.body
  end

  test "show renders for a logged-in admin with an admin panel link" do
    post session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }

    get account_path
    assert_response :success
    assert_match "Přejít do administrace", response.body
  end

  test "a logged-in customer is redirected away from the admin panel, not granted access" do
    User.create!(email_address: "zakaznik@example.com", password: "supersecret", role: "customer")
    post session_path, params: { email_address: "zakaznik@example.com", password: "supersecret" }

    get admin_root_path
    assert_redirected_to root_path

    get admin_catalog_products_path
    assert_redirected_to root_path

    get admin_orders_path
    assert_redirected_to root_path
  end

  test "show lists the current user's past orders, most recent first, but not other users' orders" do
    customer = User.create!(email_address: "majitel@example.com", password: "supersecret", role: "customer")
    other = User.create!(email_address: "jiny@example.com", password: "supersecret", role: "customer")

    older = Order.create!(order_attrs(user: customer, created_at: 2.days.ago))
    newer = Order.create!(order_attrs(user: customer, created_at: 1.day.ago))
    others_order = Order.create!(order_attrs(user: other, created_at: 1.day.ago))

    post session_path, params: { email_address: "majitel@example.com", password: "supersecret" }
    get account_path

    assert_response :success
    assert_match "č. #{newer.id}", response.body
    assert_match "č. #{older.id}", response.body
    assert_no_match "č. #{others_order.id}", response.body
    assert response.body.index(newer.id.to_s) < response.body.index(older.id.to_s)
  end

  test "reorder adds the order's items back into the cart" do
    customer = User.create!(email_address: "reorder1@example.com", password: "supersecret", role: "customer")
    variant = catalog_variants(:tramy_variant)
    order = Order.create!(order_attrs(user: customer, items: [ variant ]))

    post session_path, params: { email_address: "reorder1@example.com", password: "supersecret" }
    post reorder_order_path(order)

    assert_redirected_to cart_path
    assert_equal 1, session[:cart].size
    assert_equal variant.id, session[:cart].values.first["catalog_variant_id"]
  end

  test "reorder skips items whose product is no longer active and shows a partial notice" do
    customer = User.create!(email_address: "reorder2@example.com", password: "supersecret", role: "customer")
    active_variant = catalog_variants(:tramy_variant)
    inactive_variant = catalog_variants(:inactive_variant)
    order = Order.create!(order_attrs(user: customer, items: [ active_variant, inactive_variant ]))

    post session_path, params: { email_address: "reorder2@example.com", password: "supersecret" }
    post reorder_order_path(order)

    assert_redirected_to cart_path
    assert_equal 1, session[:cart].size
    assert_equal I18n.t("auth.reorder_partial", count: 1), flash[:notice]
  end

  test "reorder redirects back to the account with an alert when no items are available anymore" do
    customer = User.create!(email_address: "reorder3@example.com", password: "supersecret", role: "customer")
    inactive_variant = catalog_variants(:inactive_variant)
    order = Order.create!(order_attrs(user: customer, items: [ inactive_variant ]))

    post session_path, params: { email_address: "reorder3@example.com", password: "supersecret" }
    post reorder_order_path(order)

    assert_redirected_to account_path
    assert session[:cart].blank? || session[:cart].empty?
  end

  test "edit redirects to login when not authenticated" do
    get edit_account_path
    assert_redirected_to login_path
  end

  test "edit renders the profile form pre-filled with saved details" do
    customer = User.create!(
      email_address: "edit1@example.com", password: "supersecret", role: "customer",
      first_name: "Eva", billing_city: "Znojmo"
    )
    post session_path, params: { email_address: "edit1@example.com", password: "supersecret" }

    get edit_account_path
    assert_response :success
    assert_match 'value="Eva"', response.body
    assert_match 'value="Znojmo"', response.body
  end

  test "update saves the submitted profile details and redirects to the account page" do
    User.create!(email_address: "edit2@example.com", password: "supersecret", role: "customer")
    post session_path, params: { email_address: "edit2@example.com", password: "supersecret" }

    patch update_account_path, params: {
      user: {
        first_name: "Petr", last_name: "Svoboda", phone_prefix: "+420", phone_number: "123456789",
        billing_street: "Hlavní 1", billing_city: "Brno", billing_zip: "60200", billing_country: "cz",
        company_purchase: "0", delivery_address_different: "0"
      }
    }

    assert_redirected_to account_path
    customer = User.find_by(email_address: "edit2@example.com")
    assert_equal "Petr", customer.first_name
    assert_equal "Svoboda", customer.last_name
    assert_equal "Brno", customer.billing_city
    assert_equal "+420 123456789", customer.phone
  end

  test "update combines the Austrian prefix with the entered number" do
    User.create!(email_address: "edit2b@example.com", password: "supersecret", role: "customer")
    post session_path, params: { email_address: "edit2b@example.com", password: "supersecret" }

    patch update_account_path, params: { user: { phone_prefix: "+43", phone_number: "6601234567" } }

    customer = User.find_by(email_address: "edit2b@example.com")
    assert_equal "+43 6601234567", customer.phone
    assert_equal "+43", customer.phone_prefix
    assert_equal "6601234567", customer.phone_number
  end

  test "update ignores a tampered phone_prefix outside the allowed list" do
    User.create!(email_address: "edit2c@example.com", password: "supersecret", role: "customer")
    post session_path, params: { email_address: "edit2c@example.com", password: "supersecret" }

    patch update_account_path, params: { user: { phone_prefix: "+1", phone_number: "5551234" } }

    customer = User.find_by(email_address: "edit2c@example.com")
    assert_equal "+420 5551234", customer.phone
  end

  test "update cannot assign role or email_address through the profile form" do
    customer = User.create!(email_address: "edit3@example.com", password: "supersecret", role: "customer")
    post session_path, params: { email_address: "edit3@example.com", password: "supersecret" }

    patch update_account_path, params: {
      user: { first_name: "Petr", role: "admin", email_address: "hacked@example.com" }
    }

    customer.reload
    assert_equal "customer", customer.role
    assert_equal "edit3@example.com", customer.email_address
  end

  test "reorder cannot reach another user's order" do
    customer = User.create!(email_address: "reorder4@example.com", password: "supersecret", role: "customer")
    other = User.create!(email_address: "reorder5@example.com", password: "supersecret", role: "customer")
    order = Order.create!(order_attrs(user: other, items: [ catalog_variants(:tramy_variant) ]))

    post session_path, params: { email_address: "reorder4@example.com", password: "supersecret" }
    post reorder_order_path(order)

    assert_response :not_found
  end

  private

  def order_attrs(user:, created_at: Time.current, items: [])
    items_snapshot = items.map do |variant|
      {
        catalog_variant_id: variant.id,
        product_key: variant.catalog_product.key,
        image: nil,
        unit_price_czk: variant.price_czk,
        specs: [],
        quantity: 1
      }
    end.to_json

    {
      user: user,
      first_name: "Test", last_name: "Zákazník", email: user.email_address, phone: "123456789",
      billing_street: "Hlavní 1", billing_city: "Brno", billing_zip: "60200", billing_country: "cz",
      shipping_method: "pickup", payment_method: "cash",
      items_snapshot: items_snapshot,
      subtotal_czk: 1000, vat_czk: 174, total_czk: 1000,
      locale: "cs", created_at: created_at
    }
  end
end
