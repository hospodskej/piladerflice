require "test_helper"

class RegistrationsControllerTest < ActionDispatch::IntegrationTest
  test "new renders the registration form" do
    get new_registration_path
    assert_response :success
  end

  test "create registers a customer, logs them in, and redirects to the account page" do
    assert_difference "User.count", 1 do
      post registrations_path, params: { user: { email_address: "zakaznik@example.com", password: "supersecret", password_confirmation: "supersecret" } }
    end

    assert_redirected_to account_path
    assert_not_nil cookies[:session_id]
    assert_equal "customer", User.find_by(email_address: "zakaznik@example.com").role
  end

  test "create rejects a mismatched password confirmation" do
    assert_no_difference "User.count" do
      post registrations_path, params: { user: { email_address: "zakaznik@example.com", password: "supersecret", password_confirmation: "different" } }
    end

    assert_response :unprocessable_entity
  end

  test "create cannot be used to self-assign the admin role" do
    post registrations_path, params: { user: { email_address: "zakaznik@example.com", password: "supersecret", password_confirmation: "supersecret", role: "admin" } }

    assert_equal "customer", User.find_by(email_address: "zakaznik@example.com").role
  end
end
