class AddPositionToPricelistItems < ActiveRecord::Migration[8.1]
  class MigrationPricelistItem < ActiveRecord::Base
    self.table_name = "pricelist_items"
  end

  def up
    add_column :pricelist_items, :position, :integer, default: 0, null: false
    add_index :pricelist_items, [:category, :position]

    # Keep today's on-page order (insertion order) as the starting order.
    MigrationPricelistItem.reset_column_information
    MigrationPricelistItem.distinct.pluck(:category).each do |category|
      MigrationPricelistItem.where(category: category).order(:id).pluck(:id).each_with_index do |id, index|
        MigrationPricelistItem.where(id: id).update_all(position: index)
      end
    end
  end

  def down
    remove_index :pricelist_items, [:category, :position]
    remove_column :pricelist_items, :position
  end
end
