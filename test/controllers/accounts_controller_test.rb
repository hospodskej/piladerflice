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
end
