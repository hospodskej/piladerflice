class CreateOrders < ActiveRecord::Migration[8.1]
  def change
    create_table :orders do |t|
      t.string :first_name
      t.string :last_name
      t.string :email
      t.string :phone
      t.boolean :newsletter_opt_in, default: false, null: false

      t.string :billing_street
      t.string :billing_city
      t.string :billing_zip
      t.string :billing_country

      t.boolean :company_purchase, default: false, null: false
      t.string :company_name
      t.string :company_ico
      t.string :company_dic

      t.boolean :delivery_address_different, default: false, null: false
      t.string :delivery_street
      t.string :delivery_city
      t.string :delivery_zip
      t.string :delivery_country

      t.string :shipping_method
      t.string :payment_method

      t.text :items_snapshot, null: false
      t.integer :subtotal_czk, null: false
      t.integer :vat_czk, null: false
      t.integer :total_czk, null: false

      t.string :locale, null: false, default: "cs"

      t.timestamps
    end
  end
end
