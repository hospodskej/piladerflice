# How much of an order's final price is VAT, by rate, for the country the
# goods go to (VatRule). Every price here already includes VAT, so the VAT in
# an amount at rate r % is amount × r / (100 + r).
#
# A discount code and the delivery have no category of their own: they are
# split over the goods' categories in proportion to what the goods cost, so
# delivering firewood to Austria carries 13 % like the firewood itself
# ("cena dřeva + doprava").
class VatBreakdown
  attr_reader :country

  # lines: [[category, amount incl. VAT], ...] (the goods before the discount)
  def initialize(lines:, country:, discount_czk: 0, delivery_czk: 0)
    @lines = lines
    @country = VatRule.country(country)
    @discount_czk = discount_czk.to_i
    @delivery_czk = delivery_czk.to_i
  end

  # { rate in % => VAT in Kč }, highest rate first; rates with nothing are left out.
  def by_rate
    @by_rate ||= amounts_by_rate.sort_by { |rate, _| -rate }.to_h { |rate, amount| [rate, vat_in(amount, rate)] }
  end

  def vat_czk
    by_rate.values.sum
  end

  def total_czk
    goods_czk - @discount_czk + @delivery_czk
  end

  def total_excl_vat_czk
    total_czk - vat_czk
  end

  # VAT in an amount incl. VAT at the given rate.
  def self.vat_in(amount, rate)
    (amount * rate / (100.0 + rate)).round
  end

  # The price without VAT of one line incl. VAT.
  def self.excl_vat(amount, category:, country:)
    amount - vat_in(amount, VatRule.for(category).rate_for(country))
  end

  private

  def goods_czk
    @lines.sum { |_, amount| amount.to_i }
  end

  # { rate => amount incl. VAT }, the discount and the delivery spread in
  # proportion to each rate's share of the goods.
  def amounts_by_rate
    goods = Hash.new(0)
    @lines.each { |category, amount| goods[VatRule.for(category).rate_for(@country)] += amount.to_i }
    return {} if goods.empty?

    total_goods = goods.values.sum
    extra = @delivery_czk - @discount_czk
    result = {}
    allocated = 0
    goods.each_with_index do |(rate, amount), index|
      share = if index == goods.size - 1
        extra - allocated # the last rate takes the rounding remainder
      elsif total_goods.zero?
        0
      else
        (extra * amount / total_goods.to_f).round
      end
      allocated += share
      result[rate] = amount + share
    end
    result.reject { |_, amount| amount.zero? }
  end

  def vat_in(amount, rate)
    self.class.vat_in(amount, rate)
  end
end
