require "test_helper"

class InquiryTest < ActiveSupport::TestCase
  def valid_attributes
    {
      first_name: "Jana", last_name: "Nováková", email: "jana@example.com", phone: "123456789",
      items_snapshot: "[]", locale: "cs"
    }
  end

  test "valid with all required attributes" do
    assert Inquiry.new(valid_attributes).valid?
  end

  test "requires contact details" do
    inquiry = Inquiry.new(valid_attributes.except(:phone))
    assert_not inquiry.valid?
    assert inquiry.errors.of_kind?(:phone, :blank)
  end

  test "items parses the JSON snapshot" do
    inquiry = Inquiry.new(valid_attributes.merge(items_snapshot: [ { "druh" => "Dub" } ].to_json))
    assert_equal [ { "druh" => "Dub" } ], inquiry.items
  end

  test "full_name joins first and last name" do
    assert_equal "Jana Nováková", Inquiry.new(valid_attributes).full_name
  end

  test "full_address combines street/house number and zip/city, skipping blanks" do
    inquiry = Inquiry.new(valid_attributes.merge(street: "Hlavní", house_number: "12", zip: "60200", city: "Brno"))
    assert_equal "Hlavní 12, 60200 Brno", inquiry.full_address
  end

  test "full_address omits parts that are entirely blank" do
    inquiry = Inquiry.new(valid_attributes.merge(street: "Hlavní", city: "Brno"))
    assert_equal "Hlavní, Brno", inquiry.full_address
  end

  test "full_address is blank when nothing was provided" do
    assert_equal "", Inquiry.new(valid_attributes).full_address
  end
end
