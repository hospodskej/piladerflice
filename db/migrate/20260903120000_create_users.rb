class CreateUsers < ActiveRecord::Migration[8.1]
  def change
    create_table :users do |t|
      t.string :email_address, null: false
      t.string :password_digest, null: false
      # "admin" today (a single shared account); "customer" is reserved
      # for the planned future customer-login feature, so that work won't
      # need another users migration when it happens - see User#admin?.
      t.string :role, null: false, default: "admin"

      t.timestamps
    end
    add_index :users, :email_address, unique: true
  end
end
