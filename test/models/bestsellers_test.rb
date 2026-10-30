require "test_helper"

class BestsellersTest < ActiveSupport::TestCase
  setup do
    @products = CatalogProduct.where(key: %w[smrk tramy odkory]).to_a
    @template = orders(:this_month)
    Order.delete_all
  end

  def order_with(items)
    @template.dup.tap do |order|
      order.items_snapshot = items.map { |key, quantity| { product_key: key, quantity: quantity } }.to_json
      order.save!
    end
  end

  test "without orders the hand-picked order is used" do
    assert_equal({}, Bestsellers.sold_quantities)
    assert_equal %w[tramy smrk odkory], Bestsellers.sort(@products, fallback: %w[tramy smrk]).map(&:key)
  end

  test "a single order already puts what it sold first, the rest keep the hand-picked order" do
    order_with("odkory" => 2)

    assert_equal %w[odkory tramy smrk], Bestsellers.sort(@products, fallback: %w[tramy smrk]).map(&:key)
  end

  test "the products that sold most come first" do
    order_with("odkory" => 3, "smrk" => 1)
    order_with("odkory" => 2)
    4.times { order_with("smrk" => 1) }

    assert_equal({ "odkory" => 5, "smrk" => 5 }, Bestsellers.sold_quantities)
    # odkory and smrk tie, so the fallback decides; tramy sold nothing.
    assert_equal %w[smrk odkory tramy], Bestsellers.sort(@products, fallback: %w[tramy smrk]).map(&:key)
  end

  test "orders older than 3 months don't count" do
    order_with("odkory" => 1).update!(created_at: 4.months.ago)

    assert_equal({}, Bestsellers.sold_quantities)
  end
end
