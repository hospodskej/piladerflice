class RenameImageExtensionsToWebp < ActiveRecord::Migration[8.1]
  class MigrationProduct < ActiveRecord::Base
    self.table_name = "products"
  end

  class MigrationFaqItem < ActiveRecord::Base
    self.table_name = "faq_items"
  end

  class MigrationReview < ActiveRecord::Base
    self.table_name = "reviews"
  end

  class MigrationService < ActiveRecord::Base
    self.table_name = "services"
    serialize :images, type: Array, coder: JSON
  end

  EXTENSION_PATTERN = /\.(png|jpe?g)\z/i

  def up
    MigrationProduct.where.not(image: nil).find_each do |record|
      new_image = record.image.sub(EXTENSION_PATTERN, ".webp")
      record.update_column(:image, new_image) if new_image != record.image
    end

    MigrationFaqItem.where.not(image: nil).find_each do |record|
      new_image = record.image.sub(EXTENSION_PATTERN, ".webp")
      record.update_column(:image, new_image) if new_image != record.image
    end

    MigrationReview.where.not(avatar: nil).find_each do |record|
      new_avatar = record.avatar.sub(EXTENSION_PATTERN, ".webp")
      record.update_column(:avatar, new_avatar) if new_avatar != record.avatar
    end

    MigrationService.find_each do |record|
      next if record.images.blank?

      new_images = record.images.map { |path| path.sub(EXTENSION_PATTERN, ".webp") }
      record.update_column(:images, new_images) if new_images != record.images
    end
  end

  def down
    raise ActiveRecord::IrreversibleMigration
  end
end
