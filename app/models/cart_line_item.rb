class CartLineItem
  attr_reader :id, :product_key, :image, :unit_price_czk, :specs, :quantity, :category

  def initialize(id, attrs)
    @id = id
    @product_key = attrs["product_key"]
    @image = attrs["image"]
    @unit_price_czk = attrs["unit_price_czk"].to_i
    @specs = attrs["specs"] || []
    @quantity = attrs["quantity"].to_i
    # The product's category, for its VAT rate. Carts from before it was
    # stored look it up.
    @category = attrs["category"].presence || CatalogProduct.find_by(key: @product_key)&.category
  end

  def line_total_czk
    unit_price_czk * quantity
  end
end
