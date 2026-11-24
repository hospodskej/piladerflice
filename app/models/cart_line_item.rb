class CartLineItem
  attr_reader :id, :product_key, :image, :unit_price_czk, :specs, :quantity, :category

  def initialize(id, attrs)
    @id = id
    @product_key = attrs["product_key"]
    @image = attrs["image"]
    @unit_price_czk = attrs["unit_price_czk"].to_i
    @unit_price_eur = attrs["unit_price_eur"].presence && BigDecimal(attrs["unit_price_eur"].to_s)
    @specs = attrs["specs"] || []
    @quantity = attrs["quantity"].to_i
    # The product's category, for its VAT rate. Carts from before it was
    # stored look it up.
    @category = attrs["category"].presence || CatalogProduct.find_by(key: @product_key)&.category
  end

  def line_total_czk
    unit_price_czk * quantity
  end

  # In Kč and € (the € price typed in the admin, or converted).
  def unit_price
    Money.from(czk: unit_price_czk, eur: @unit_price_eur)
  end

  def line_total
    unit_price * quantity
  end
end
