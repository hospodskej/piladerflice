class AddThemeToUsers < ActiveRecord::Migration[8.1]
  def change
    # "light" or "dark"; nil until the customer first picks one while signed in
    add_column :users, :theme, :string
  end
end
