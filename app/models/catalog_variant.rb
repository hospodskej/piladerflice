class CatalogVariant < ApplicationRecord
  include Translatable

  belongs_to :catalog_product

  translates :variant_label

  validates :key, presence: true, uniqueness: { scope: :catalog_product_id }
  validates :price_czk, numericality: { greater_than: 0 }
  validates :variant_group, inclusion: { in: %w[kontejner bedny] }, allow_nil: true

  scope :ordered, -> { order(:position, :id) }

  def to_cart_specs
    specs = []

    case catalog_product.template
    when "firewood"
      variant_key = variant_group == "bedny" ? "common.stacked_variant" : "common.loose_variant"
      specs << { "type" => "key", "value" => variant_key }
      specs << { "type" => "raw", "value" => length_label } if length_label.present?
      specs << { "type" => "amount", "value" => amount_value, "unit_key" => "common.per_m3" } if amount_value.present?
    when "lumber"
      specs << { "type" => "bilingual", "value" => variant_label, "value_de" => variant_label_de }
      specs << { "type" => "raw", "value" => length_label } if length_label.present?
    when "simple_variant"
      specs << { "type" => "bilingual", "value" => variant_label, "value_de" => variant_label_de }
    end

    specs
  end
end
