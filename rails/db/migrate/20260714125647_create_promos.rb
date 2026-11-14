class CreatePromos < ActiveRecord::Migration[8.1]
  def change
    create_table :promos do |t|
      t.string :title
      t.string :feature_1
      t.string :feature_2
      t.string :feature_3
      t.string :price
      t.string :image
      t.string :link

      t.timestamps
    end
  end
end
