class AddCheckoutProfileFieldsToUsers < ActiveRecord::Migration[8.1]
  def change
    add_column :users, :first_name, :string
    add_column :users, :last_name, :string
    add_column :users, :phone, :string
    add_column :users, :billing_street, :string
    add_column :users, :billing_city, :string
    add_column :users, :billing_zip, :string
    add_column :users, :billing_country, :string
    add_column :users, :company_purchase, :boolean, default: false, null: false
    add_column :users, :company_name, :string
    add_column :users, :company_ico, :string
    add_column :users, :company_dic, :string
    add_column :users, :delivery_address_different, :boolean, default: false, null: false
    add_column :users, :delivery_street, :string
    add_column :users, :delivery_city, :string
    add_column :users, :delivery_zip, :string
    add_column :users, :delivery_country, :string
  end
end
