class CreateDiscountCodes < ActiveRecord::Migration[8.1]
  def change
    create_table :discount_codes do |t|
      t.string :code, null: false
      t.string :kind, null: false, default: "percent"
      t.integer :value, null: false
      t.boolean :active, null: false, default: true
      t.date :valid_until
      t.integer :usage_limit
      t.integer :times_used, null: false, default: 0
      t.integer :min_order_czk
      t.string :note
      t.timestamps
    end
    add_index :discount_codes, :code, unique: true

    add_column :orders, :discount_code, :string
    add_column :orders, :discount_czk, :integer
  end
end
