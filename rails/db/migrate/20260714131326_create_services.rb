class CreateServices < ActiveRecord::Migration[8.1]
  def change
    create_table :services do |t|
      t.string :title
      t.text :content
      t.text :images
      t.string :button_text
      t.string :button_path

      t.timestamps
    end
  end
end
