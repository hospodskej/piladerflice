# An amount in both currencies the shop shows: Kč for the Czech site and €
# for the Austrian one (/at). The € amount is the one Štěpán typed in the
# admin when there is one (exact), otherwise today's exchange rate applied
# to the Kč amount ("≈", not exact). Sums stay exact only if every part was.
class Money
  attr_reader :czk, :eur

  def self.zero
    new(czk: 0, eur: 0, exact: true)
  end

  # An amount in Kč, with the € amount typed in the admin (or nil to convert).
  def self.from(czk:, eur: nil)
    eur.nil? ? new(czk: czk, eur: convert(czk), exact: false) : new(czk: czk, eur: eur, exact: true)
  end

  def self.convert(czk)
    (BigDecimal(czk.to_i.to_s) / BigDecimal(ExchangeRateService.czk_per_eur.to_s)).round(2)
  end

  def initialize(czk:, eur:, exact:)
    @czk = czk.to_i
    @eur = BigDecimal(eur.to_s).round(2)
    @exact = exact
  end

  def exact_eur?
    @exact
  end

  def +(other)
    Money.new(czk: czk + other.czk, eur: eur + other.eur, exact: exact_eur? && other.exact_eur?)
  end

  def -(other)
    Money.new(czk: czk - other.czk, eur: eur - other.eur, exact: exact_eur? && other.exact_eur?)
  end

  def *(factor)
    Money.new(czk: czk * factor, eur: eur * factor, exact: exact_eur?)
  end

  def zero?
    czk.zero? && eur.zero?
  end

  def ==(other)
    other.is_a?(Money) && czk == other.czk && eur == other.eur
  end

  def self.sum(amounts)
    amounts.reduce(zero) { |total, amount| total + amount }
  end
end
