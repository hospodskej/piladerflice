module CartHelper
  def cart_line_description(item)
    parts = [catalog_product_title(item.product_key)]
    parts.concat(item.specs.map { |spec| cart_spec_text(spec) })
    parts.join(", ")
  end

  # Kč on the Czech site, € on the Austrian one. Takes a Money (the € price
  # from the admin) or a number of Kč (converted at today's rate).
  def cart_price(amount)
    ExchangeRateService.display_amount(amount)
  end

  # A line's price without VAT, in both currencies.
  def money_excl_vat(money, category:, country:)
    Money.new(
      czk: VatBreakdown.excl_vat(money.czk, category: category, country: country),
      eur: VatBreakdown.excl_vat(money.eur, category: category, country: country, precision: 2),
      exact: money.exact_eur?
    )
  end

  # { rate => Money } from the Kč and the € VAT split of the same goods.
  def vat_money_by_rate(vat_czk, vat_eur, exact:)
    rates = (vat_czk.by_rate.keys | vat_eur.by_rate.keys).sort.reverse
    rates.to_h do |rate|
      [rate, Money.new(czk: vat_czk.by_rate.fetch(rate, 0), eur: vat_eur.by_rate.fetch(rate, 0), exact: exact)]
    end
  end

  # Why a discount code can't be used, for the customer.
  def discount_reason_message(reason, discount)
    t("cart.discount_errors.#{reason}", amount: discount&.min_order_czk && cart_price(discount.min_order_czk))
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

