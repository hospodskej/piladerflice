require "test_helper"

class Admin::DashboardControllerTest < ActionDispatch::IntegrationTest
  setup do
    post admin_session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }
  end

  test "shows total, this-month, and last-month order counts" do
    get admin_root_path
    assert_response :success

    assert_select ".admin-dashboard-card-number", text: Order.count.to_s
    assert_select ".admin-dashboard-card-number", text: Order.where(created_at: Time.current.beginning_of_month..).count.to_s
    assert_select ".admin-dashboard-card-number",
      text: Order.where(created_at: 1.month.ago.beginning_of_month..1.month.ago.end_of_month).count.to_s
  end

  test "this-month and last-month counts only include orders from their respective month" do
    get admin_root_path

    assert_select ".admin-dashboard-card-label", text: "objednávky tento měsíc"
    assert_select ".admin-dashboard-card-label", text: "objednávky minulý měsíc"

    assert_equal 1, Order.where(created_at: Time.current.beginning_of_month..).count
    assert_equal 2, Order.where(created_at: 1.month.ago.beginning_of_month..1.month.ago.end_of_month).count
  end
end
