require "test_helper"

class FaqItemTest < ActiveSupport::TestCase
  test "title_i18n falls back to the Czech title when there is no German translation" do
    faq_item = faq_items(:one)
    I18n.with_locale(:de) { assert_equal faq_item.title, faq_item.title_i18n }
  end
end
