class CreateInquiries < ActiveRecord::Migration[8.1]
  def change
    create_table :inquiries do |t|
      t.string :first_name
      t.string :last_name
      t.string :email
      t.string :phone

      t.string :street
      t.string :house_number
      t.string :city
      t.string :zip

      t.text :items_snapshot, null: false

      t.text :notes

      t.string :locale, null: false, default: "cs"

      t.timestamps
    end
  end
end
