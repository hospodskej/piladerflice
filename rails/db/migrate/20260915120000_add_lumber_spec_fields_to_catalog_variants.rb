class AddLumberSpecFieldsToCatalogVariants < ActiveRecord::Migration[8.1]
  def change
    add_column :catalog_variants, :grade, :string
    add_column :catalog_variants, :grade_de, :string
    add_column :catalog_variants, :width_mm, :integer
    add_column :catalog_variants, :height_mm, :integer
  end
end
