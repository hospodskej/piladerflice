# "Nejprodávanější": products ranked by how much of them was ordered over
# the last 3 months. Products that sold nothing yet keep the hand-picked
# order below the ones that did.
module Bestsellers
  PERIOD = 3.months

  # Product key => quantity ordered in the last 3 months.
  def self.sold_quantities
    Order.where(created_at: PERIOD.ago..).pluck(:items_snapshot).each_with_object(Hash.new(0)) do |snapshot, sold|
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
