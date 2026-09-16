require "test_helper"

class ApplicationHelperTest < ActionView::TestCase
  test "image_tag_if_attached renders nothing for an unattached image" do
    product = catalog_products(:odkory)

    assert_nil image_tag_if_attached(product.image, alt: "test")
  end

  test "image_tag_if_attached renders nothing for a pending, not-yet-saved attachment" do
    product = catalog_products(:tramy)
    product.image = { io: StringIO.new("fake image bytes"), filename: "tramy.png", content_type: "image/png" }

    assert_not product.image.blob.persisted?
    assert_nil image_tag_if_attached(product.image, alt: "test")
  end

  test "image_tag_if_attached renders the image once the attachment is saved" do
    product = catalog_products(:tramy)
    product.image.attach(
      io: File.open(Rails.root.join("app/assets/images/eshop/tramy.png")),
      filename: "tramy.png"
    )

    assert product.image.blob.persisted?
    assert_match "<img", image_tag_if_attached(product.image, alt: "test")
  end

  test "image_tag_with_fallback uses the fallback asset when nothing is attached" do
    product = catalog_products(:odkory)

    assert_match "eshop/container", image_tag_with_fallback(product.image, "eshop/container.png", alt: "test")
  end
end
