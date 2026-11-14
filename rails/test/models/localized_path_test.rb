require "test_helper"

class LocalizedPathTest < ActiveSupport::TestCase
  test "Czech paths stay as they are" do
    assert_equal "/kontakt", LocalizedPath.call("/kontakt", "cs")
    assert_equal "/eshop?category=rezivo", LocalizedPath.call("/eshop?category=rezivo", :cs)
    assert_equal "/", LocalizedPath.call("/", "cs")
  end

  test "German paths get the /at prefix, keeping query and anchor" do
    assert_equal "/at/kontakt#cenik", LocalizedPath.call("/kontakt#cenik", "de")
    assert_equal "/at/eshop?category=rezivo&sort=x", LocalizedPath.call("/eshop?category=rezivo&sort=x", :de)
    assert_equal "/at/sortiment/palivove-drevo", LocalizedPath.call("/sortiment/palivove-drevo", "de")
  end

  test "the home page is /at, not /at/" do
    assert_equal "/at", LocalizedPath.call("/", "de")
    assert_equal "/at?x=1", LocalizedPath.call("/?x=1", "de")
  end

  test "switching language replaces the prefix instead of stacking it" do
    assert_equal "/at/kontakt", LocalizedPath.call("/at/kontakt", "de")
    assert_equal "/kontakt", LocalizedPath.call("/at/kontakt", "cs")
    assert_equal "/", LocalizedPath.call("/at", "cs")
    assert_equal "/?x=1", LocalizedPath.call("/at?x=1", "cs")
  end

  test "a path that merely starts with the letters de isn't treated as a prefix" do
    assert_equal "/at/delivery", LocalizedPath.call("/delivery", "de")
    assert_equal "/delivery", LocalizedPath.call("/delivery", "cs")
    assert_equal "/dekujeme", LocalizedPath.call("/dekujeme", "cs")
  end

  test "anchors, external links and blanks are left alone" do
    assert_equal "#", LocalizedPath.call("#", "de")
    assert_equal "https://example.com/a", LocalizedPath.call("https://example.com/a", "de")
    assert_equal "mailto:a@b.cz", LocalizedPath.call("mailto:a@b.cz", "de")
    assert_equal "", LocalizedPath.call(nil, "de")
  end

  test "strip" do
    assert_equal "/kontakt", LocalizedPath.strip("/at/kontakt")
    assert_equal "/", LocalizedPath.strip("/at")
    assert_equal "/", LocalizedPath.strip("/")
  end

  test "the German site is the Austrian one: prefix /at, never /de" do
    assert_equal "/at/kontakt", LocalizedPath.call("/kontakt", "de")
    assert_equal "/kontakt", LocalizedPath.call("/de/kontakt", "cs").sub(%r{\A/de/}, "/"), "a stray /de prefix is just another path"
    assert_equal "de", LocalizedPath.locale_for("at")
    assert_nil LocalizedPath.locale_for("de")
    assert_nil LocalizedPath.locale_for(nil)
    assert_equal "at", LocalizedPath.segment_for(:de)
    assert_nil LocalizedPath.segment_for(:cs)
  end
end
