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

  test "registering an e-mail that already has an account doesn't say so; the owner gets a mail instead" do
    User.create!(email_address: "uz-mam@example.com", password: "supersecret", role: "customer")

    assert_emails 1 do
      post registrations_path, params: { user: { email_address: "Uz-Mam@example.com", password: "jineheslo1", password_confirmation: "jineheslo1" } }
    end

    assert_response :unprocessable_entity
    assert_select ".page-flash-alert li", text: /Tato kombinace e-mailu a hesla není správná/
    assert_no_match(/už je použitý|je již zaregistrován|already/, response.body)
    mail = ActionMailer::Base.deliveries.last
    assert_equal ["uz-mam@example.com"], mail.to
    assert_includes mail.text_part.body.to_s, "/zapomenute-heslo"
  end

  test "other mistakes in the registration are still shown as they are" do
    post registrations_path, params: { user: { email_address: "novy@example.com", password: "kratke", password_confirmation: "kratke" } }
    assert_response :unprocessable_entity
    assert_select ".page-flash-alert li", text: /Heslo/
    assert_no_emails { }
  end
end
