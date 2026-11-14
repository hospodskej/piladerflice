class CreateReviews < ActiveRecord::Migration[8.1]
  def change
    create_table :reviews do |t|
      t.string :name, null: false
      t.integer :stars, null: false, default: 5
      t.text :text, null: false
      t.date :reviewed_on, null: false
      t.integer :position, default: 0, null: false

      t.timestamps
    end

    add_index :reviews, :position
  end
end
