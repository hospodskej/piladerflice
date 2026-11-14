class AddCategoryToInquiries < ActiveRecord::Migration[8.1]
  def change
    add_column :inquiries, :category, :string, null: false, default: "palivove"
  end
end
