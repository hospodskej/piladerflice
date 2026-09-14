require "test_helper"

class PromoTest < ActiveSupport::TestCase
  test "title_i18n falls back to the Czech title when there is no German translation" do
    promo = promos(:one)
    I18n.with_locale(:de) { assert_equal promo.title, promo.title_i18n }
  end
end
