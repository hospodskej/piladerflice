require "test_helper"

class PagesControllerTest < ActionDispatch::IntegrationTest
  test "should get terms" do
    get terms_url
    assert_response :success
  end

  test "should get cookies policy" do
    get cookies_policy_url
    assert_response :success
  end

  test "should get privacy policy in both languages" do
    get privacy_policy_url
    assert_response :success
    assert_select "h1", "Zásady ochrany osobních údajů"
    assert_select ".legal-content h2", minimum: 5
    assert_select ".legal-content", /45665451/

    get privacy_policy_url(locale: "de")
    assert_response :success
    assert_select "h1", "Datenschutzerklärung"
    assert_select ".legal-content", /45665451/
  end

  test "footer links to the privacy policy" do
    get root_url
    assert_select "footer a[href=?]", privacy_policy_path
  end

  test "contact form and registration link to the privacy policy" do
    get kontakt_url
    assert_select ".consent-text a[href=?]", privacy_policy_path
    get new_registration_url
    assert_select ".auth-privacy-note a[href=?]", privacy_policy_path
  end
end
