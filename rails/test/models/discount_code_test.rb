require "test_helper"

class DiscountCodeTest < ActiveSupport::TestCase
  test "the code is stored in capitals without spaces and looked up the same way" do
    code = DiscountCode.create!(code: " jaro10 ", kind: "percent", value: 10)

    assert_equal "JARO10", code.code
    assert_equal code, DiscountCode.lookup("Jaro10")
    assert_nil DiscountCode.lookup("")
  end

  test "codes must be unique whatever their case, and percentages at most 100" do
    DiscountCode.create!(code: "JARO10", kind: "percent", value: 10)

    assert_not DiscountCode.new(code: "jaro10", kind: "percent", value: 10).valid?
    assert_not DiscountCode.new(code: "VELKA", kind: "percent", value: 101).valid?
    assert DiscountCode.new(code: "VELKA", kind: "fixed", value: 101).valid?
    assert_not DiscountCode.new(code: "S MEZEROU", kind: "fixed", value: 100).valid?
  end

  test "a percentage is taken off the goods, a fixed amount never more than the goods cost" do
    assert_equal 850, DiscountCode.new(kind: "percent", value: 10).amount_for(8_500)
    assert_equal 500, DiscountCode.new(kind: "fixed", value: 500).amount_for(8_500)
    assert_equal 300, DiscountCode.new(kind: "fixed", value: 500).amount_for(300)
  end

  test "why a code can't be used" do
    code = DiscountCode.new(code: "X", kind: "fixed", value: 100)
    assert_nil code.unusable_reason(1_000)

    assert_equal :inactive, DiscountCode.new(active: false).unusable_reason(1_000)
    assert_equal :expired, DiscountCode.new(valid_until: Date.yesterday).unusable_reason(1_000)
    assert_nil DiscountCode.new(valid_until: Date.current).unusable_reason(1_000)
    assert_equal :used_up, DiscountCode.new(usage_limit: 2, times_used: 2).unusable_reason(1_000)
    assert_equal :min_order, DiscountCode.new(min_order_czk: 2_000).unusable_reason(1_000)
  end

  test "redeeming counts a use, but not past the limit" do
    code = DiscountCode.create!(code: "JEDNOU", kind: "fixed", value: 100, usage_limit: 1)

    assert code.redeem!
    assert_not code.redeem!
    assert_equal 1, code.reload.times_used
  end
end
