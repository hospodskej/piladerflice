class ChangeUsersRoleDefaultToCustomer < ActiveRecord::Migration[8.1]
  def change
    change_column_default :users, :role, from: "admin", to: "customer"
  end
end
