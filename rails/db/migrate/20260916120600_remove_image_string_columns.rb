class RemoveImageStringColumns < ActiveRecord::Migration[8.1]
  def change
    remove_column :catalog_products, :image, :string
    remove_column :promos, :image, :string
  end
end
