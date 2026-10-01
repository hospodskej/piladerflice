require "test_helper"

class SessionsControllerTest < ActionDispatch::IntegrationTest
  setup do
    @customer = User.create!(email_address: "zakaznik@example.com", password: "supersecret", role: "customer")
  end

  test "new renders the login form" do
    get login_path
    assert_response :success
  end

  test "create logs a customer in and redirects to the account page" do
    post session_path, params: { email_address: "zakaznik@example.com", password: "supersecret" }

    assert_redirected_to account_path
    assert_not_nil cookies[:session_id]
  end

  test "create logs an admin in through the same form" do
    post session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }

    assert_redirected_to account_path
    assert_not_nil cookies[:session_id]
  end

  test "an admin who logged in through the customer form can then reach the admin panel" do
    post session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }

    get admin_root_path
    assert_response :success
  end

  test "create rejects a wrong password" do
    post session_path, params: { email_address: "zakaznik@example.com", password: "wrongpassword" }

    assert_response :unprocessable_entity
    assert_nil cookies[:session_id]
  end

  test "destroy logs the user out" do
    post session_path, params: { email_address: "zakaznik@example.com", password: "supersecret" }
    delete logout_path

    assert_redirected_to root_path
    get account_path
    assert_redirected_to login_path
  end
end
