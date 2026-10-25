require "test_helper"

class Admin::DiscountCodesControllerTest < ActionDispatch::IntegrationTest
  setup do
    post session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }
  end

  test "the admin creates a code and sees it in the list" do
    post admin_discount_codes_path, params: { discount_code: { code: "leto", kind: "fixed", value: "300", min_order_czk: "5000", usage_limit: "10", active: "1" } }

    assert_redirected_to admin_discount_codes_path
    code = DiscountCode.last
    assert_equal ["LETO", "fixed", 300, 5_000, 10], [code.code, code.kind, code.value, code.min_order_czk, code.usage_limit]

    get admin_discount_codes_path
    assert_select "td", text: /LETO/
    assert_select "td", text: "300 Kč"
    assert_select "td", text: "0 / 10×"
  end

  test "the new and edit forms render" do
    get new_admin_discount_code_path
    assert_select "input[name='discount_code[code]']"

    get edit_admin_discount_code_path(DiscountCode.create!(code: "JARO10", kind: "percent", value: 10))
    assert_select "input[name='discount_code[value]'][value='10']"
  end

  test "an invalid code is refused" do
    post admin_discount_codes_path, params: { discount_code: { code: "", kind: "percent", value: "150" } }

    assert_response :unprocessable_entity
    assert_select ".admin-flash-alert"
  end

  test "a code can be switched off and deleted" do
    code = DiscountCode.create!(code: "JARO10", kind: "percent", value: 10)

    patch admin_discount_code_path(code), params: { discount_code: { active: "0" } }
    assert_not code.reload.active?
    get admin_discount_codes_path
    assert_select "td", text: /Vypnutý/

    delete admin_discount_code_path(code)
    assert_not DiscountCode.exists?(code.id)
  end

  test "guests can't reach the codes" do
    delete logout_path
    get admin_discount_codes_path
    assert_response :redirect
  end
end
