# How much of an order's final price is VAT, by rate, for the country the
# goods go to (VatRule). Every price here already includes VAT, so the VAT in
# an amount at rate r % is amount × r / (100 + r).
#
# A discount code and the delivery have no category of their own: they are
# split over the goods' categories in proportion to what the goods cost, so
# delivering firewood to Austria carries 13 % like the firewood itself
# ("cena dřeva + doprava").
#
# Works in whole Kč (precision 0, the default) or in € to the cent
# (precision 2, for orders on the Austrian site).
class VatBreakdown
  attr_reader :country

  # lines: [[category, amount incl. VAT], ...] (the goods before the discount)
  def initialize(lines:, country:, discount_czk: 0, delivery_czk: 0, precision: 0)
    @precision = precision
    @lines = lines.map { |category, amount| [category, number(amount)] }
    @country = VatRule.country(country)
    @discount = number(discount_czk)
    @delivery = number(delivery_czk)
  end

  # { rate in % => VAT }, highest rate first; rates with nothing are left out.
  def by_rate
    @by_rate ||= amounts_by_rate.sort_by { |rate, _| -rate }.to_h { |rate, amount| [rate, vat_in(amount, rate)] }
  end

  def vat_czk
    by_rate.values.sum(number(0))
  end
  alias_method :vat, :vat_czk

  def total_czk
    goods - @discount + @delivery
  end
  alias_method :total, :total_czk

  def total_excl_vat_czk
    total_czk - vat_czk
  end
  alias_method :total_excl_vat, :total_excl_vat_czk

  # VAT in an amount incl. VAT at the given rate (whole Kč unless a precision is given).
  def self.vat_in(amount, rate, precision: 0)
    value = (BigDecimal(amount.to_s) * rate / (100 + rate)).round(precision)
    precision.zero? ? value.to_i : value
  end

  # The price without VAT of one line incl. VAT.
  def self.excl_vat(amount, category:, country:, precision: 0)
    amount - vat_in(amount, VatRule.for(category).rate_for(country), precision: precision)
  end

  private

  def number(value)
    rounded = BigDecimal(value.to_s).round(@precision)
    @precision.zero? ? rounded.to_i : rounded
  end

  def goods
    @lines.sum(number(0)) { |_, amount| amount }
  end

  # { rate => amount incl. VAT }, the discount and the delivery spread in
  # proportion to each rate's share of the goods.
  def amounts_by_rate
    goods_by_rate = Hash.new(number(0))
    @lines.each { |category, amount| goods_by_rate[VatRule.for(category).rate_for(@country)] += amount }
    return {} if goods_by_rate.empty?

    total_goods = goods_by_rate.values.sum
    extra = @delivery - @discount
    result = {}
    allocated = number(0)
    goods_by_rate.each_with_index do |(rate, amount), index|
      share = if index == goods_by_rate.size - 1
        extra - allocated # the last rate takes the rounding remainder
      elsif total_goods.zero?
        number(0)
      else
        number(BigDecimal(extra.to_s) * amount / total_goods)
      end
      allocated += share
      result[rate] = amount + share
    end
    result.reject { |_, amount| amount.zero? }
  end

  def vat_in(amount, rate)
    self.class.vat_in(amount, rate, precision: @precision)
  end
end
