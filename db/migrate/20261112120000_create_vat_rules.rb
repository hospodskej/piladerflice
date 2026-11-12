# VAT by product category and destination country (Štěpán, 12 Nov 2026):
# - Czech Republic: 21 % on everything.
# - Austria: firewood, štěpka, piliny (and the other by-products) 13 %,
#   stavební řezivo and kamenivo 20 %.
# Firewood prices are entered in the admin with VAT, stavební řezivo and the
# by-products without it ("21 % + DPH"). A customer pays the same final price
# in both countries; only how much of it is VAT differs.
class CreateVatRules < ActiveRecord::Migration[8.1]
  RULES = [
    { category: "palivove", prices_include_vat: true, rate_cz: 21, rate_at: 13 },
    { category: "rezivo", prices_include_vat: false, rate_cz: 21, rate_at: 20 },
    { category: "zbytky", prices_include_vat: false, rate_cz: 21, rate_at: 13 },
    { category: "kamenivo", prices_include_vat: true, rate_cz: 21, rate_at: 20 }
  ].freeze

  def up
    create_table :vat_rules do |t|
      t.string :category, null: false
      t.boolean :prices_include_vat, null: false, default: true
      t.integer :rate_cz, null: false, default: 21
      t.integer :rate_at, null: false, default: 20
      t.timestamps
    end
    add_index :vat_rules, :category, unique: true

    now = Time.current
    VatRuleRow.insert_all!(RULES.map { |rule| rule.merge(created_at: now, updated_at: now) })

    # The VAT split of an order as it was when ordered: { "13" => 1234, "20" => 56 }
    # (rate in % => VAT in Kč), and the country whose rates were used.
    add_column :orders, :vat_breakdown, :json
    add_column :orders, :vat_country, :string
  end

  def down
    remove_column :orders, :vat_country
    remove_column :orders, :vat_breakdown
    drop_table :vat_rules
  end

  class VatRuleRow < ActiveRecord::Base
    self.table_name = "vat_rules"
  end
end
