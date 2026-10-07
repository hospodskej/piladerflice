module ApplicationHelper
  # Sets the page <title> and <meta name="description"> from the `meta.<key>`
  # translations (or from explicit values for dynamic pages such as products).
  # Pages without a description (cart, account...) only get a title.
  def page_meta(key, title: nil, description: nil, brand_only_title: false)
    title ||= t("meta.#{key}.title")
    description ||= I18n.t("meta.#{key}.description", default: nil)

    content_for :title, (brand_only_title ? title : "#{title} | #{t('site.name')}")
    if description.present?
      content_for :description, description.to_s.squish.truncate(160, separator: " ")
      @indexable_page = true
    end
    nil
  end

  # The signed-in user's profile picture. The checksum in the URL changes
  # whenever the picture does, so browsers can cache it for a long time.
  def avatar_image_tag(user, **options)
    return unless user&.avatar&.attached?

    image_tag avatar_path(v: user.avatar.blob.checksum.first(12)), alt: t("auth.avatar_alt"), **options
  end

  def image_tag_if_attached(attachment, **options)
    return unless attachment&.attached?
    return unless attachment.blob&.persisted?

    # locale: nil keeps the language out of the image address (storage URLs
    # aren't language-specific and shouldn't get "?locale=de").
    image_tag(rails_storage_redirect_path(attachment.blob, locale: nil), **options)
  end

  def image_tag_with_fallback(attachment, fallback, **options)
    tag = image_tag_if_attached(attachment, **options)
    return tag if tag
    return image_tag(fallback, **options) if fallback.is_a?(String)

    image_tag_if_attached(fallback, **options)
  end
end
