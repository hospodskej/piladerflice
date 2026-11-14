class OrderMailer < ApplicationMailer
  NOTIFICATION_RECIPIENT = ENV.fetch("ORDER_NOTIFICATION_EMAIL", "pavelpatockaa@gmail.com")

  def new_order(order)
    @order = order
    @lines = line_summaries(order, locale: :cs)

    mail(
      to: NOTIFICATION_RECIPIENT,
      subject: "Nová objednávka ##{order.id} – #{order.full_name}"
    )
  end

  def customer_confirmation(order)
    @order = order
    @locale = order.locale.to_sym
    @lines = line_summaries(order, locale: @locale)

    subject = @locale == :de ? "Bestellbestätigung Nr. #{order.id} – Pila Derflice" : "Potvrzení objednávky č. #{order.id} – Pila Derflice"

    mail(to: order.email, subject: subject)
  end

  private

  def line_summaries(order, locale:)
    order.items.map do |item|
      line_item = CartLineItem.new(SecureRandom.uuid, item)
      {
        description: line_description(line_item, locale: locale),
        quantity: line_item.quantity,
        unit_price: ExchangeRateService.display_amount(line_item.unit_price_czk, locale: locale),
        line_total: ExchangeRateService.display_amount(line_item.line_total_czk, locale: locale)
      }
    end
  end

  def line_description(item, locale:)
    parts = [catalog_product_title(item.product_key, locale: locale)]
    parts.concat(item.specs.map { |spec| spec_text(spec, locale: locale) })
    parts.join(", ")
  end

  def catalog_product_title(product_key, locale:)
    product = CatalogProduct.find_by(key: product_key)
    if product
      locale.to_sym == :de ? (product.title_de.presence || product.title) : product.title
    else
      I18n.t("eshop.products.#{product_key}.title", locale: locale)
    end
  end

  def spec_text(spec, locale:)
    case spec["type"]
    when "key"
      I18n.t(spec["value"], locale: locale)
    when "amount"
      "#{spec["value"]} #{I18n.t(spec["unit_key"], locale: locale)}"
    when "bilingual"
      locale.to_sym == :de ? (spec["value_de"].presence || spec["value"]) : spec["value"]
    else
      spec["value"].to_s
    end
  end
end
