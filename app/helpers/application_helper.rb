module ApplicationHelper
  def image_tag_if_attached(attachment, **options)
    return unless attachment&.attached?
    return unless attachment.blob&.persisted?

    image_tag(attachment, **options)
  end

  def image_tag_with_fallback(attachment, fallback, **options)
    tag = image_tag_if_attached(attachment, **options)
    return tag if tag
    return image_tag(fallback, **options) if fallback.is_a?(String)

    image_tag_if_attached(fallback, **options)
  end
end
