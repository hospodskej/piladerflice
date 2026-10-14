require "test_helper"

class ApplicationHelperTest < ActionView::TestCase
  test "error_count uses the Czech plural" do
    assert_equal "1 chyba", error_count(1)
    assert_equal "3 chyby", error_count(3)
    assert_equal "5 chyb", error_count(5)
  end

  test "image_tag_if_attached renders nothing for an unattached image" do
    product = catalog_products(:odkory)

    assert_nil image_tag_if_attached(product.image, alt: "test")
  end

  test "image_tag_if_attached renders nothing for a pending, not-yet-saved attachment" do
    product = catalog_products(:tramy)
    product.image = { io: StringIO.new("fake image bytes"), filename: "tramy.webp", content_type: "image/webp" }

    assert_not product.image.blob.persisted?
    assert_nil image_tag_if_attached(product.image, alt: "test")
  end

  test "image_tag_if_attached renders the image once the attachment is saved" do
    product = catalog_products(:tramy)
    product.image.attach(
      io: File.open(Rails.root.join("app/assets/images/eshop/tramy.webp")),
      filename: "tramy.webp"
    )

    assert product.image.blob.persisted?
    assert_match "<img", image_tag_if_attached(product.image, alt: "test")
  end

  test "image_tag_with_fallback uses the fallback asset when nothing is attached" do
    product = catalog_products(:odkory)

    assert_match "eshop/container", image_tag_with_fallback(product.image, "eshop/container.webp", alt: "test")
  end
end
