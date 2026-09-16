module ApplicationHelper
  def image_tag_if_attached(attachment, **options)
    image_tag(attachment, **options) if attachment&.attached?
  end

  def image_tag_with_fallback(attachment, fallback, **options)
    return image_tag(attachment, **options) if attachment&.attached?
    return image_tag(fallback, **options) if fallback.is_a?(String)

    image_tag_if_attached(fallback, **options)
  end
end
