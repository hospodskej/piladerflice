class CartLineItem
  attr_reader :id, :product_key, :image, :unit_price_czk, :specs, :quantity

  def initialize(id, attrs)
    @id = id
    @product_key = attrs["product_key"]
    @image = attrs["image"]
    @unit_price_czk = attrs["unit_price_czk"].to_i
    @specs = attrs["specs"] || []
    @quantity = attrs["quantity"].to_i
  end

  def line_total_czk
    unit_price_czk * quantity
  end
end
