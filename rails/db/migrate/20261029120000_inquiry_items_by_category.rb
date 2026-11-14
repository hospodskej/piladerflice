# An inquiry can ask for firewood and lumber together: items_snapshot becomes
# { "palivove" => [rows], "stavebni" => [rows] } instead of one category's
# rows next to a category column.
class InquiryItemsByCategory < ActiveRecord::Migration[8.1]
  class Inquiry < ActiveRecord::Base
    self.table_name = "inquiries"
  end

  def up
    Inquiry.reset_column_information
    Inquiry.find_each do |inquiry|
      rows = JSON.parse(inquiry.items_snapshot)
      next unless rows.is_a?(Array)

      inquiry.update_columns(items_snapshot: { inquiry.category => rows }.to_json)
    end
    remove_column :inquiries, :category
  end

  def down
    add_column :inquiries, :category, :string, null: false, default: "palivove"
    Inquiry.reset_column_information
    Inquiry.find_each do |inquiry|
      by_category = JSON.parse(inquiry.items_snapshot)
      next unless by_category.is_a?(Hash) && by_category.any?

      category, rows = by_category.first
      inquiry.update_columns(category: category, items_snapshot: rows.to_json)
    end
  end
end
