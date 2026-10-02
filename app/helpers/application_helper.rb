module ApplicationHelper
  def image_tag_if_attached(attachment, **options)
    return unless attachment&.attached?
    return unless attachment.blob&.persisted?

    image_tag(webp_variant(attachment), **options)
  end

  def image_tag_with_fallback(attachment, fallback, **options)
    tag = image_tag_if_attached(attachment, **options)
    return tag if tag
    return image_tag(fallback, **options) if fallback.is_a?(String)

    image_tag_if_attached(fallback, **options)
  end

  private

  # Serves a WebP-converted variant so uploaded images (whatever format
  # they were uploaded in) match the WebP used across the rest of the
  # site. SVGs are left alone since there's no raster format to gain here.
  def webp_variant(attachment)
    return attachment if attachment.blob.content_type == "image/svg+xml"

    attachment.variant(format: :webp)
  end
end
