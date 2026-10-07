module ApplicationHelper
  # The signed-in user's profile picture. The checksum in the URL changes
  # whenever the picture does, so browsers can cache it for a long time.
  def avatar_image_tag(user, **options)
    return unless user&.avatar&.attached?

    image_tag avatar_path(v: user.avatar.blob.checksum.first(12)), alt: t("auth.avatar_alt"), **options
  end

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
