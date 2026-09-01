class CreateOrders < ActiveRecord::Migration[8.1]
  def change
    create_table :orders do |t|
      # Personal details
      t.string :first_name
      t.string :last_name
      t.string :email
      t.string :phone
      t.boolean :newsletter_opt_in, default: false, null: false

      # Billing address
      t.string :billing_street
      t.string :billing_city
      t.string :billing_zip
      t.string :billing_country

      # Optional company details ("Nakupuji na firmu")
      t.boolean :company_purchase, default: false, null: false
      t.string :company_name
      t.string :company_ico
      t.string :company_dic

      # Optional separate delivery address ("Doručovací adresa, pokud je
      # rozdílná od fakturační") - only used when delivery_address_different
      t.boolean :delivery_address_different, default: false, null: false
      t.string :delivery_street
      t.string :delivery_city
      t.string :delivery_zip
      t.string :delivery_country

      # "doprava" / "address" (delivery to address, individually quoted) or
      # "pickup" (personal pickup at the warehouse, free)
      t.string :shipping_method
      # "cash" (cash on pickup) or "bank_transfer" (proforma invoice)
      t.string :payment_method

      # A snapshot of the cart at checkout time, since Cart itself is
      # session-only and would otherwise be lost the moment the session's
      # cart is cleared after a successful order. JSON-serialized array of
      # the same shape as Cart's session-stored line items (see
      # app/models/cart.rb / cart_line_item.rb).
      t.text :items_snapshot, null: false
      t.integer :subtotal_czk, null: false
      t.integer :vat_czk, null: false
      t.integer :total_czk, null: false

      # Which language the order was placed in, so the confirmation email
      # can be worded to match what the customer actually saw.
      t.string :locale, null: false, default: "cs"

      t.timestamps
    end
  end
end
