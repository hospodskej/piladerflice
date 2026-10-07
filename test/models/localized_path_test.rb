require "test_helper"

class LocalizedPathTest < ActiveSupport::TestCase
  test "Czech paths stay as they are" do
    assert_equal "/kontakt", LocalizedPath.call("/kontakt", "cs")
    assert_equal "/eshop?category=rezivo", LocalizedPath.call("/eshop?category=rezivo", :cs)
    assert_equal "/", LocalizedPath.call("/", "cs")
  end

  test "German paths get the /de prefix, keeping query and anchor" do
    assert_equal "/de/kontakt#cenik", LocalizedPath.call("/kontakt#cenik", "de")
    assert_equal "/de/eshop?category=rezivo&sort=x", LocalizedPath.call("/eshop?category=rezivo&sort=x", :de)
    assert_equal "/de/sortiment/palivove-drevo", LocalizedPath.call("/sortiment/palivove-drevo", "de")
  end

  test "the home page is /de, not /de/" do
    assert_equal "/de", LocalizedPath.call("/", "de")
    assert_equal "/de?x=1", LocalizedPath.call("/?x=1", "de")
  end

  test "switching language replaces the prefix instead of stacking it" do
    assert_equal "/de/kontakt", LocalizedPath.call("/de/kontakt", "de")
    assert_equal "/kontakt", LocalizedPath.call("/de/kontakt", "cs")
    assert_equal "/", LocalizedPath.call("/de", "cs")
    assert_equal "/?x=1", LocalizedPath.call("/de?x=1", "cs")
  end

  test "a path that merely starts with the letters de isn't treated as a prefix" do
    assert_equal "/de/delivery", LocalizedPath.call("/delivery", "de")
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
    assert_equal "/kontakt", LocalizedPath.strip("/de/kontakt")
    assert_equal "/", LocalizedPath.strip("/de")
    assert_equal "/", LocalizedPath.strip("/")
  end
end
