module ApplicationHelper
  def image_tag_if_attached(attachment, **options)
    image_tag(attachment, **options) if attachment&.attached?
  end
end
