# "Nejprodávanější": products ranked by how much of them was ordered over
# the last year. Until the shop has a few orders, the hand-picked fallback
# order is used instead, so one order doesn't reshuffle the whole list.
module Bestsellers
  PERIOD = 1.year
  MIN_ORDERS = 5

  # Product key => quantity ordered; empty while there are too few orders.
  def self.sold_quantities
    orders = Order.where(created_at: PERIOD.ago..)
    return {} if orders.count < MIN_ORDERS

    orders.pluck(:items_snapshot).each_with_object(Hash.new(0)) do |snapshot, sold|
      JSON.parse(snapshot).each { |item| sold[item["product_key"]] += item["quantity"].to_i }
    rescue JSON::ParserError
      next
    end
  end

  # Best sellers first; products that sold the same keep the fallback order,
  # then the catalogue order.
  def self.sort(products, fallback:, sold: sold_quantities)
    products.sort_by { |p| [-sold.fetch(p.key, 0), fallback.index(p.key) || fallback.size, p.position, p.id] }
  end
end
