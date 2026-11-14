class AddStackedToCatalogVariants < ActiveRecord::Migration[8.1]
  class MigrationVariant < ActiveRecord::Base
    self.table_name = "catalog_variants"
  end

  # Firewood in a container can be loose ("sypané": 25 and 33 cm) or laid in
  # neat rows ("rovnané": 50 cm and 1 m), so "container" no longer means
  # "loose". The flag says which one a variant is; crates are always stacked.
  def up
    add_column :catalog_variants, :stacked, :boolean, default: false, null: false

    MigrationVariant.reset_column_information
    MigrationVariant.where(variant_group: "bedny").update_all(stacked: true)
    MigrationVariant.where(variant_group: "kontejner").find_each do |variant|
      variant.update_columns(stacked: true) if length_in_cm(variant.length_label).to_i >= 50
    end
  end

  def down
    remove_column :catalog_variants, :stacked
  end

  private

  # "100 cm" -> 100, "1 m" -> 100, anything else -> nil
  def length_in_cm(label)
    number = label.to_s[/\d+(?:[.,]\d+)?/]&.tr(",", ".")&.to_f
    return nil unless number

    label.to_s.match?(/\bcm\b/i) ? number : (number * 100 if label.to_s.match?(/\bm\b/i))
  end
end
