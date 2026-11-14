require "test_helper"

class Admin::PricelistItemsControllerTest < ActionDispatch::IntegrationTest
  setup do
    post session_path, params: { email_address: "admin@piladerflice.cz", password: "correct-horse-battery" }
  end

  test "index shows one editable section per ceník table" do
    get admin_pricelist_items_path
    assert_response :success
    PricelistItem::CATEGORIES.each { |category| assert_select "div##{category}" }
    assert_select "#stavebni input[name='pricelist_item[price]'][value=?]", pricelist_items(:fosny).price
  end

  test "kamenivo has no add row once its single price exists" do
    get admin_pricelist_items_path
    assert_select "#kamenivo input[type=submit][value='+ Přidat']", count: 0
    assert_select "#zbytky input[type=submit][value='+ Přidat']", count: 1
  end

  test "create appends the item to the end of its category" do
    assert_difference -> { PricelistItem.count }, 1 do
      post admin_pricelist_items_path, params: { pricelist_item: { category: "stavebni", item_name: "Hranoly", details: "3m", price: "8 000 Kč / m³" } }
    end

    item = PricelistItem.last
    assert_equal 2, item.position
    assert_equal %w[Fošny Trámy Hranoly], PricelistItem.where(category: "stavebni").ordered.map(&:item_name)
    assert_redirected_to admin_pricelist_items_path(anchor: "stavebni")
  end

  test "create with a missing price explains the error in Czech" do
    assert_no_difference -> { PricelistItem.count } do
      post admin_pricelist_items_path, params: { pricelist_item: { category: "zbytky", item_name: "Kůra" } }
    end
    assert_equal "Cena je povinné pole", flash[:alert]
  end

  test "update changes the price but not the category" do
    item = pricelist_items(:fosny)
    patch admin_pricelist_item_path(item), params: { pricelist_item: { price: "9 900 Kč / m³", category: "zbytky" } }

    assert_redirected_to admin_pricelist_items_path(anchor: "stavebni")
    item.reload
    assert_equal "9 900 Kč / m³", item.price
    assert_equal "stavebni", item.category
  end

  test "destroy removes the item" do
    assert_difference -> { PricelistItem.count }, -1 do
      delete admin_pricelist_item_path(pricelist_items(:tramy))
    end
  end

  test "move_up and move_down reorder items within their category" do
    patch move_up_admin_pricelist_item_path(pricelist_items(:tramy))
    assert_equal %w[Trámy Fošny], PricelistItem.where(category: "stavebni").ordered.map(&:item_name)

    patch move_down_admin_pricelist_item_path(pricelist_items(:tramy))
    assert_equal %w[Fošny Trámy], PricelistItem.where(category: "stavebni").ordered.map(&:item_name)
  end

  test "moving works even when items share a position" do
    pricelist_items(:tramy).update_column(:position, 0)
    first, second = PricelistItem.where(category: "stavebni").ordered.to_a

    patch move_up_admin_pricelist_item_path(second)
    assert_equal [second, first], PricelistItem.where(category: "stavebni").ordered.to_a
  end

  test "move_up does nothing for the first item" do
    patch move_up_admin_pricelist_item_path(pricelist_items(:fosny))
    assert_equal %w[Fošny Trámy], PricelistItem.where(category: "stavebni").ordered.map(&:item_name)
  end

  test "logged-out visitors are sent to the login page" do
    delete logout_path
    get admin_pricelist_items_path
    assert_redirected_to login_path
  end

  test "logged-in customers can't edit the ceník" do
    delete logout_path
    User.create!(email_address: "zakaznik@example.com", password: "zakaznik-heslo-123", role: "customer")
    post session_path, params: { email_address: "zakaznik@example.com", password: "zakaznik-heslo-123" }

    patch admin_pricelist_item_path(pricelist_items(:fosny)), params: { pricelist_item: { price: "1 Kč" } }
    assert_redirected_to root_path
    assert_equal "9 800 Kč / m<sup>3</sup>", pricelist_items(:fosny).reload.price
  end
end
