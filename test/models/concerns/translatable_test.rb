require "test_helper"

class TranslatableTest < ActiveSupport::TestCase
  test "uses the German column for the German locale when present" do
    product = catalog_products(:smrk)
    I18n.with_locale(:de) { assert_equal "Fichte", product.title_i18n }
  end

  test "falls back to the Czech column when the German one is blank" do
    product = catalog_products(:tramy)
    product.update!(type_label_de: nil)
    I18n.with_locale(:de) { assert_equal product.type_label, product.type_label_i18n }
  end

  test "always uses the Czech column for the Czech locale" do
    product = catalog_products(:smrk)
    I18n.with_locale(:cs) { assert_equal product.title, product.title_i18n }
  end
end
