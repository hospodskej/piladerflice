require "test_helper"

class Admin::OrdersControllerTest < ActionDispatch::IntegrationTest
  setup do
    post session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }
  end

  test "index lists all orders with a delete button" do
    get admin_orders_path
    assert_response :success
    assert_select "a", text: "##{orders(:this_month).id}"
    assert_select "form[action=?]", admin_order_path(orders(:this_month))
  end

  test "show renders an order's details" do
    get admin_order_path(orders(:this_month))
    assert_response :success
    assert_select "h1", text: "Objednávka ##{orders(:this_month).id}"
  end

  test "show lists the approximate delivery price of an order delivered to an address" do
    order = orders(:this_month)
    order.update!(shipping_method: "address", delivery_distance_km: 100, delivery_czk: 4000)

    get admin_order_path(order)
    assert_select ".admin-order-totals p", text: "Doprava (100 km tam i zpět, orientačně): 4 000 Kč"
  end

  test "destroy removes the order and redirects to the index" do
    order = orders(:this_month)

    assert_difference -> { Order.count }, -1 do
      delete admin_order_path(order)
    end

    assert_redirected_to admin_orders_path
    assert_not Order.exists?(order.id)
  end

  test "requires admin authentication" do
    delete logout_path

    get admin_orders_path
    assert_redirected_to login_path

    delete admin_order_path(orders(:this_month))
    assert_redirected_to login_path
    assert Order.exists?(orders(:this_month).id)
  end
end
