class CreateProducts < ActiveRecord::Migration[8.1]
  def change
    create_table :products do |t|
      t.string :title
      t.text :text
      t.string :image
      t.string :link

      t.timestamps
    end
  end
end
