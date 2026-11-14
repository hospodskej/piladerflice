require "test_helper"

class SettingTest < ActiveSupport::TestCase
  test "the delivery price is 40 Kč per km until the admin changes it" do
    assert_equal 40, Setting.delivery_czk_per_km

    Setting.create!(key: "delivery_czk_per_km", value: "45")
    assert_equal 45, Setting.delivery_czk_per_km
  end

  test "the delivery price must be a whole number from 1 to 1000" do
    %w[0 -5 1001 4.5 abc].push("").each do |value|
      assert_not Setting.new(key: "delivery_czk_per_km", value: value).valid?, "#{value.inspect} should be refused"
    end
    assert Setting.new(key: "delivery_czk_per_km", value: "1000").valid?
  end

  test "only known settings can be stored" do
    assert_not Setting.new(key: "something_else", value: "1").valid?
  end
end
