require "test_helper"

class PasswordsControllerTest < ActionDispatch::IntegrationTest
  setup do
    @user = User.create!(email_address: "zapomnel@example.com", password: "stareheslo1", role: "customer")
  end

  def reset_link
    ActionMailer::Base.deliveries.last.text_part.body.to_s[%r{http://\S+nove-heslo\?token=\S+}]
  end

  test "the login page links to the forgotten password page" do
    get login_path
    assert_select "a[href='#{new_password_path}']", text: "Zapomněli jste heslo?"
  end

  test "the answer is the same for a registered and an unknown e-mail; only the account gets a mail" do
    assert_emails 1 do
      post passwords_path, params: { email_address: " Zapomnel@Example.com " }
    end
    known = [response.location, flash[:notice]]

    assert_no_emails do
      post passwords_path, params: { email_address: "nikdo@example.com" }
    end
    assert_equal known, [response.location, flash[:notice]]
    assert_equal "zapomnel@example.com", ActionMailer::Base.deliveries.last.to.first
  end

  test "the link sets a new password, signs the account out everywhere and works only once" do
    post session_path, params: { email_address: @user.email_address, password: "stareheslo1" }
    assert_equal 1, @user.sessions.count
    reset!

    post passwords_path, params: { email_address: @user.email_address }
    link = reset_link
    assert link

    get link
    # The token leaves the address bar straight away.
    assert_redirected_to edit_password_path
    follow_redirect!
    assert_select "p", text: /zapomnel@example.com/

    patch password_path, params: { password: "noveheslo22", password_confirmation: "noveheslo22" }
    assert_redirected_to login_path
    assert_equal 0, @user.sessions.count
    assert @user.reload.authenticate("noveheslo22")

    get link
    assert_redirected_to new_password_path
    assert_equal "Odkaz pro nastavení hesla už neplatí. Platí 15 minut a jen jednou – nechte si poslat nový.", flash[:alert]
  end

  test "the link expires after 15 minutes" do
    post passwords_path, params: { email_address: @user.email_address }
    link = reset_link

    travel 16.minutes do
      get link
      assert_redirected_to new_password_path
    end
  end

  test "a new password must be at least 8 characters and match the confirmation" do
    post passwords_path, params: { email_address: @user.email_address }
    get reset_link

    patch password_path, params: { password: "kratke", password_confirmation: "kratke" }
    assert_response :unprocessable_entity
    patch password_path, params: { password: "noveheslo22", password_confirmation: "jine-heslo" }
    assert_response :unprocessable_entity
    patch password_path, params: { password: "", password_confirmation: "" }
    assert_response :unprocessable_entity
    assert @user.reload.authenticate("stareheslo1")
  end

  test "without a valid link there's no form" do
    get edit_password_path
    assert_redirected_to new_password_path

    patch password_path, params: { token: "forged", password: "noveheslo22", password_confirmation: "noveheslo22" }
    assert_redirected_to new_password_path
    assert @user.reload.authenticate("stareheslo1")
  end

  test "reset requests are limited" do
    assert_emails 5 do
      6.times { post passwords_path, params: { email_address: @user.email_address } }
    end
    assert_equal I18n.t("security.too_many_attempts", locale: :cs), flash[:alert]
  end

  test "the German site sends a German mail with a German link" do
    post passwords_path(locale: "at"), params: { email_address: @user.email_address }
    mail = ActionMailer::Base.deliveries.last
    assert_equal "Neues Passwort – Pila Derflice", mail.subject
    assert_includes mail.text_part.body.to_s, "/at/nove-heslo?token="
  end
end
