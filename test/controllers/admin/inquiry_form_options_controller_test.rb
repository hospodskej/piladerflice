require "test_helper"

class Admin::InquiryFormOptionsControllerTest < ActionDispatch::IntegrationTest
  setup do
    post admin_session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }
  end

  test "index lists options grouped by category and field" do
    get admin_inquiry_form_options_path
    assert_response :success
    assert_select "h2", text: "Varianta"
    assert_select "h2", text: "Položka"
  end

  test "create adds a new option at the end of its group" do
    assert_difference -> { InquiryFormOption.count }, 1 do
      post admin_inquiry_form_options_path, params: { inquiry_form_option: { category: "palivove", field: "varianta", value: "Nová možnost", value_de: "Neue Option" } }
    end

    option = InquiryFormOption.last
    assert_equal "palivove", option.category
    assert_equal "varianta", option.field
    assert_equal 2, option.position
    assert_redirected_to admin_inquiry_form_options_path(anchor: "palivove-varianta")
  end

  test "create rejects a field that doesn't belong to the category" do
    assert_no_difference -> { InquiryFormOption.count } do
      post admin_inquiry_form_options_path, params: { inquiry_form_option: { category: "palivove", field: "polozka", value: "X" } }
    end
    assert_redirected_to admin_inquiry_form_options_path
  end

  test "update changes an option's value" do
    option = inquiry_form_options(:palivove_varianta_skladane)
    patch admin_inquiry_form_option_path(option), params: { inquiry_form_option: { value: "Skládané (upraveno)" } }

    assert_redirected_to admin_inquiry_form_options_path(anchor: "palivove-varianta")
    assert_equal "Skládané (upraveno)", option.reload.value
  end

  test "destroy removes an option" do
    option = inquiry_form_options(:palivove_varianta_sypane)
    assert_difference -> { InquiryFormOption.count }, -1 do
      delete admin_inquiry_form_option_path(option)
    end
  end

  test "move_up swaps position with the previous option" do
    first = inquiry_form_options(:palivove_varianta_skladane)
    second = inquiry_form_options(:palivove_varianta_sypane)

    patch move_up_admin_inquiry_form_option_path(second)

    assert_equal 0, second.reload.position
    assert_equal 1, first.reload.position
  end

  test "move_up does nothing for the first option in its group" do
    first = inquiry_form_options(:palivove_varianta_skladane)
    patch move_up_admin_inquiry_form_option_path(first)

    assert_equal 0, first.reload.position
  end

  test "move_down swaps position with the next option" do
    first = inquiry_form_options(:palivove_varianta_skladane)
    second = inquiry_form_options(:palivove_varianta_sypane)

    patch move_down_admin_inquiry_form_option_path(first)

    assert_equal 1, first.reload.position
    assert_equal 0, second.reload.position
  end

  test "requires admin authentication" do
    delete admin_session_path
    get admin_inquiry_form_options_path
    assert_redirected_to new_admin_session_path
  end
end
