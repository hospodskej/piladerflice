class AddAvatarToReviews < ActiveRecord::Migration[8.1]
  def change
    add_column :reviews, :avatar, :string
  end
end
