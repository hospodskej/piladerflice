class CatalogProduct < ApplicationRecord
  include Translatable

  TEMPLATES = %w[firewood lumber simple_variant].freeze
  CATEGORIES = %w[palivove rezivo zbytky kamenivo].freeze
  HARDNESS_VALUES = %w[hard soft].freeze

  has_many :catalog_variants, -> { order(:position, :id) }, dependent: :destroy

  translates :title, :type_label, :subtitle, :description, :drying_note, :image_alt

  validates :key, presence: true, uniqueness: true
  validates :template, inclusion: { in: TEMPLATES }
  validates :category, inclusion: { in: CATEGORIES }
  validates :hardness, inclusion: { in: HARDNESS_VALUES }, allow_nil: true

  scope :active, -> { where(active: true) }
  scope :ordered, -> { order(:position, :id) }

  def starting_price_czk
    (catalog_variants.where(in_stock: true).order(:price_czk).first || catalog_variants.order(:price_czk).first)&.price_czk
  end
end
