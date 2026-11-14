require "test_helper"

class ProductTest < ActiveSupport::TestCase
  test "title_i18n falls back to the Czech title when there is no German translation" do
    product = products(:one)
    I18n.with_locale(:de) { assert_equal product.title, product.title_i18n }
  end
end
