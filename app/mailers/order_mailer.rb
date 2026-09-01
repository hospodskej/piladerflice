# There's no payment gateway or admin order-management screen in this app -
# this email (together with the Order record it's built from) is the whole
# notification mechanism a new order relies on. Always written in Czech and
# priced in CZK regardless of which language/currency the customer shopped
# in, since it's going to the business owner for fulfillment, not the
# customer - see CheckoutController#confirm for where this gets triggered.
class OrderMailer < ApplicationMailer
  NOTIFICATION_RECIPIENT = ENV.fetch("ORDER_NOTIFICATION_EMAIL", "pavelpatockaa@gmail.com")

  def new_order(order)
    @order = order
    # Built here rather than left to the view/helpers, so this email is
    # guaranteed correct (Czech wording, CZK prices) no matter what locale
    # happens to be active when it's triggered - see #line_summaries.
    @lines = line_summaries(order)

    mail(
      to: NOTIFICATION_RECIPIENT,
      subject: "Nová objednávka ##{order.id} – #{order.full_name}"
    )
  end

  private

  def line_summaries(order)
    order.items.map do |item|
      line_item = CartLineItem.new(SecureRandom.uuid, item)
      {
        description: line_description(line_item),
        quantity: line_item.quantity,
        unit_price: czk(line_item.unit_price_czk),
        line_total: czk(line_item.line_total_czk)
      }
    end
  end

  def line_description(item)
    parts = [I18n.t("eshop.products.#{item.product_key}.title", locale: :cs)]
    parts.concat(item.specs.map { |spec| spec_text(spec) })
    parts.join(", ")
  end

  def spec_text(spec)
    case spec["type"]
    when "key"
      I18n.t(spec["value"], locale: :cs)
    when "amount"
      "#{spec["value"]} #{I18n.t(spec["unit_key"], locale: :cs)}"
    else
      spec["value"].to_s
    end
  end

  def czk(amount)
    ExchangeRateService.display_amount(amount, locale: :cs)
  end
end
