class AddGermanTranslations < ActiveRecord::Migration[8.1]
  def change
    add_column :products, :title_de, :string
    add_column :products, :text_de, :text

    add_column :pricelist_items, :item_name_de, :string
    add_column :pricelist_items, :details_de, :string
    add_column :pricelist_items, :price_de, :string
    add_column :pricelist_items, :subcategory_de, :string

    add_column :promos, :title_de, :string
    add_column :promos, :feature_1_de, :string
    add_column :promos, :feature_2_de, :string
    add_column :promos, :feature_3_de, :string
    add_column :promos, :price_de, :string

    add_column :services, :title_de, :string
    add_column :services, :content_de, :text
    add_column :services, :button_text_de, :string

    add_column :faq_items, :title_de, :string
    add_column :faq_items, :content_de, :text
  end
end
