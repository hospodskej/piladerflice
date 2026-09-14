require "test_helper"

class InquiryFormOptionTest < ActiveSupport::TestCase
  test "for_field returns options for a category/field in position order" do
    values = InquiryFormOption.for_field("palivove", "varianta").pluck(:value)
    assert_equal [ "Skládané", "Sypané" ], values
  end

  test "value_i18n falls back to the Czech value when no German translation is set" do
    option = inquiry_form_options(:stavebni_polozka_tram)
    option.update!(value_de: nil)

    I18n.with_locale(:de) { assert_equal "Trám", option.value_i18n }
  end

  test "value_i18n uses the German value when present" do
    option = inquiry_form_options(:stavebni_polozka_tram)
    I18n.with_locale(:de) { assert_equal "Balken", option.value_i18n }
  end

  test "requires the field to belong to the category" do
    option = InquiryFormOption.new(category: "palivove", field: "polozka", value: "X")
    assert_not option.valid?
    assert option.errors.of_kind?(:field, :invalid_for_category)
  end

  test "accepts a field that does belong to the category" do
    option = InquiryFormOption.new(category: "stavebni", field: "polozka", value: "Hranol")
    assert option.valid?
  end

  test "requires a category from the known list" do
    option = InquiryFormOption.new(category: "bogus", field: "varianta", value: "X")
    assert_not option.valid?
    assert option.errors.of_kind?(:category, :inclusion)
  end

  test "requires a value" do
    option = InquiryFormOption.new(category: "palivove", field: "varianta")
    assert_not option.valid?
    assert option.errors.of_kind?(:value, :blank)
  end

  test "ROW_FIELDS includes mnozstvi for stavebni even though it has no dropdown options" do
    assert_not_includes InquiryFormOption::DROPDOWN_FIELDS["stavebni"], "mnozstvi"
    assert_includes InquiryFormOption::ROW_FIELDS["stavebni"], "mnozstvi"
  end
end
