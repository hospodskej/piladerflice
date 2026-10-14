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
    return product.title_i18n if product

    # A product no longer sold (an old order): its old translated title, or
    # else its key made readable, rather than "Translation missing".
    t("eshop.products.#{product_key}.title", default: product_key.to_s.humanize)
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

