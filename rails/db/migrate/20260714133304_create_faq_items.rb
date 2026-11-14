class CreateFaqItems < ActiveRecord::Migration[8.1]
  def change
    create_table :faq_items do |t|
      t.string :title
      t.text :content
      t.string :image

      t.timestamps
    end
  end
end
