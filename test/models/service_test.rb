require "test_helper"

class ServiceTest < ActiveSupport::TestCase
  test "title_i18n uses the German title when present" do
    service = services(:one)
    I18n.with_locale(:de) { assert_equal "Lieferung", service.title_i18n }
  end

  test "images deserializes as an array of paths" do
    assert_equal [ "sluzby/doprava1.png", "sluzby/doprava2.png" ], services(:one).images
  end
end
