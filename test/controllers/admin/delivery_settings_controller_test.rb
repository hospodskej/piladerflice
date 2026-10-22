require "test_helper"

class Admin::DeliverySettingsControllerTest < ActionDispatch::IntegrationTest
  setup do
    post session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }
  end

  test "the admin sees the current price per km" do
    get edit_admin_delivery_settings_path

    assert_response :success
    assert_select "input[name='setting[value]'][value='40']"
  end

  test "a new price is saved and shows up in the e-shop and the terms" do
    patch admin_delivery_settings_path, params: { setting: { value: "55" } }

    assert_redirected_to edit_admin_delivery_settings_path
    assert_equal 55, Setting.delivery_czk_per_km

    get eshop_path
    assert_select ".delivery-note", text: /aktuálně 55 Kč za km/
    get terms_path
    assert_includes response.body, "cena dopravy činí 55 Kč"
  end

  test "a price that isn't a whole number from 1 to 1000 is refused" do
    patch admin_delivery_settings_path, params: { setting: { value: "0" } }

    assert_response :unprocessable_entity
    assert_select ".admin-flash-alert"
    assert_equal 40, Setting.delivery_czk_per_km
  end

  test "only an admin can change it" do
    delete logout_path
    patch admin_delivery_settings_path, params: { setting: { value: "1" } }

    assert_equal 40, Setting.delivery_czk_per_km
  end
end
