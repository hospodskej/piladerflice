require "test_helper"

# Every literal translation key the app uses must exist in Czech and German.
# (A missing one shows "Translation missing: ..." to visitors, and a test
# that compares against I18n.t(...) of the same key can't notice.)
class TranslationKeysTest < ActiveSupport::TestCase
  KEY_PATTERN = /\bt\(\s*["']([a-z0-9_]+(?:\.[a-z0-9_]+)+)["']/

  test "every literal key used in app/ exists in cs and de" do
    keys = Dir[Rails.root.join("app/**/*.{rb,erb}")].flat_map { |file| File.read(file).scan(KEY_PATTERN).flatten }.uniq
    assert_operator keys.size, :>, 100

    missing = keys.product(%i[cs de]).reject { |key, locale| I18n.exists?(key, locale) }
    assert_empty missing.map { |key, locale| "#{locale}.#{key}" }
  end
end
