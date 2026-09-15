class CatalogVariant < ApplicationRecord
  include Translatable

  belongs_to :catalog_product

  translates :variant_label
  translates :grade

  validates :key, presence: true, uniqueness: { scope: :catalog_product_id }
  validates :price_czk, numericality: { greater_than: 0 }
  validates :variant_group, inclusion: { in: %w[kontejner bedny] }, allow_nil: true

  scope :ordered, -> { order(:position, :id) }

  def dimension_filter_key
    return nil unless width_mm.present? && height_mm.present?

    "#{width_mm}x#{height_mm}"
  end

  def grade_filter_key
    grade.presence && grade.parameterize
  end

  def to_cart_specs
    specs = []

    case catalog_product.template
    when "firewood"
      variant_key = variant_group == "bedny" ? "common.stacked_variant" : "common.loose_variant"
      specs << { "type" => "key", "value" => variant_key }
      specs << { "type" => "raw", "value" => length_label } if length_label.present?
      specs << { "type" => "amount", "value" => amount_value, "unit_key" => "common.per_m3" } if amount_value.present?
    when "lumber"
      specs << { "type" => "bilingual", "value" => grade, "value_de" => grade_de } if grade.present?
      specs << { "type" => "raw", "value" => length_label } if length_label.present?
      specs << { "type" => "raw", "value" => "#{width_mm}×#{height_mm} mm" } if width_mm.present? && height_mm.present?
    when "simple_variant"
      specs << { "type" => "bilingual", "value" => variant_label, "value_de" => variant_label_de }
    end

    specs
  end
end
