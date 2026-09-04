# Two emails come out of a completed checkout - both built here since
# there's no payment gateway or admin order-management screen in this app,
# so these emails (together with the Order record they're built from) are
# the entire notification mechanism a new order relies on:
#
#   - #new_order: to the business, always in Czech/CZK regardless of which
#     language the customer shopped in, since it's for fulfillment.
#   - #customer_confirmation: to the customer, in whichever language/
#     currency they actually used at checkout (order.locale).
#
# Both build fully-rendered description/price strings directly in this
# class rather than relying on view helpers, so the correct wording and
# currency are guaranteed regardless of whatever locale happens to be
# ambient when a mailer job runs.
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
    parts = [I18n.t("eshop.products.#{item.product_key}.title", locale: locale)]
    parts.concat(item.specs.map { |spec| spec_text(spec, locale: locale) })
    parts.join(", ")
  end

  def spec_text(spec, locale:)
    case spec["type"]
    when "key"
      I18n.t(spec["value"], locale: locale)
    when "amount"
      "#{spec["value"]} #{I18n.t(spec["unit_key"], locale: locale)}"
    else
      spec["value"].to_s
    end
  end
end
