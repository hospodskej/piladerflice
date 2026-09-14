class CreateInquiryFormOptions < ActiveRecord::Migration[8.1]
  def change
    create_table :inquiry_form_options do |t|
      t.string :category, null: false
      t.string :field, null: false
      t.string :value, null: false
      t.string :value_de
      t.integer :position, default: 0, null: false

      t.timestamps
    end

    add_index :inquiry_form_options, [ :category, :field, :position ]
  end
end
