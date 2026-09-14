require "test_helper"

class Admin::SessionsControllerTest < ActionDispatch::IntegrationTest
  test "new renders the login form" do
    get new_admin_session_path
    assert_response :success
  end

  test "create logs the admin in with correct credentials and redirects to the dashboard" do
    post admin_session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }

    assert_redirected_to admin_root_path
    assert_not_nil cookies[:session_id]
  end

  test "create is case-insensitive and trims whitespace on the email address" do
    post admin_session_path, params: { email_address: "  ADMIN@PilaDerflice.cz  ", password: "correct-horse-battery" }
    assert_redirected_to admin_root_path
  end

  test "create rejects a wrong password" do
    post admin_session_path, params: { email_address: "admin@piladerflice.cz", password: "wrong-password" }

    assert_response :unprocessable_entity
    assert_nil cookies[:session_id]
  end

  test "create rejects an unknown email address" do
    post admin_session_path, params: { email_address: "nobody@piladerflice.cz", password: "correct-horse-battery" }
    assert_response :unprocessable_entity
  end

  test "admin pages redirect to login when not authenticated" do
    get admin_root_path
    assert_redirected_to new_admin_session_path
  end

  test "destroy logs the admin out" do
    post admin_session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }
    delete admin_session_path

    assert_redirected_to new_admin_session_path
    get admin_root_path
    assert_redirected_to new_admin_session_path
  end
end
