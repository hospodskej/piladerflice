module CartHelper
  def cart_line_description(item)
    parts = [catalog_product_title(item.product_key)]
    parts.concat(item.specs.map { |spec| cart_spec_text(spec) })
    parts.join(", ")
  end

  def cart_price(amount_czk)
    ExchangeRateService.display_amount(amount_czk)
  end

  private

  def catalog_product_title(product_key)
    product = CatalogProduct.find_by(key: product_key)
    product ? product.title_i18n : t("eshop.products.#{product_key}.title")
  end

  def cart_spec_text(spec)
    case spec["type"]
    when "key"
      t(spec["value"])
    when "amount"
      "#{spec["value"]} #{t(spec["unit_key"])}"
    when "bilingual"
      I18n.locale == :de ? (spec["value_de"].presence || spec["value"]) : spec["value"]
    else
      spec["value"].to_s
    end
  end
end

