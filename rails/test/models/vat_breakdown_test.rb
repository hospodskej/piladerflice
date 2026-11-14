require "test_helper"

class VatBreakdownTest < ActiveSupport::TestCase
  test "Czech VAT is 21 % of everything" do
    vat = VatBreakdown.new(lines: [ [ "palivove", 12_100 ], [ "rezivo", 1_210 ] ], country: "cz")

    assert_equal({ 21 => 2_310 }, vat.by_rate)
    assert_equal 13_310, vat.total_czk
    assert_equal 11_000, vat.total_excl_vat_czk
  end

  test "in Austria each category has its rate and the delivery is split by the goods' share" do
    # firewood 11 300 (13 %) and lumber 12 000 (20 %), delivery 2 330
    vat = VatBreakdown.new(lines: [ [ "palivove", 11_300 ], [ "rezivo", 12_000 ] ], country: "at", delivery_czk: 2_330)

    firewood_share = (2_330 * 11_300 / 23_300.0).round # 1 130
    assert_equal({ 20 => VatBreakdown.vat_in(12_000 + 2_330 - firewood_share, 20),
                   13 => VatBreakdown.vat_in(11_300 + firewood_share, 13) }, vat.by_rate)
    assert_equal 25_630, vat.total_czk
  end

  test "a discount lowers the VAT in proportion" do
    vat = VatBreakdown.new(lines: [ [ "palivove", 11_300 ] ], country: "at", discount_czk: 1_130)

    assert_equal({ 13 => VatBreakdown.vat_in(10_170, 13) }, vat.by_rate)
    assert_equal 10_170, vat.total_czk
  end

  test "an unknown country counts as the Czech Republic" do
    assert_equal "cz", VatBreakdown.new(lines: [], country: "de").country
  end
end
