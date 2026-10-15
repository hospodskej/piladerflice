class CatalogProduct < ApplicationRecord
  include Translatable

  TEMPLATES = %w[firewood lumber simple_variant].freeze
  CATEGORIES = %w[palivove rezivo zbytky kamenivo].freeze
  HARDNESS_VALUES = %w[hard soft].freeze

  has_many :catalog_variants, -> { order(:position, :id) }, dependent: :destroy
  has_one_attached :image
  include OptimizesUploadedImage

  translates :title, :type_label, :subtitle, :description, :drying_note, :image_alt

  normalizes :hardness, with: ->(hardness) { hardness.presence }

  # An emptied "Pořadí" field means the default order, not NULL (the column
  # doesn't allow NULL and saving failed with an error page).
  normalizes :position, with: ->(position) { position || 0 }, apply_to_nil: true

  validates :key, presence: true, uniqueness: true
  validates :template, inclusion: { in: TEMPLATES }
  validates :category, inclusion: { in: CATEGORIES }
  validates :hardness, inclusion: { in: HARDNESS_VALUES }, allow_nil: true

  # The "from" price: the cheapest variant in stock, or the cheapest of all
  # when none is in stock (nil without variants).
  STARTING_PRICE_SQL = <<~SQL.squish
    (SELECT COALESCE(MIN(CASE WHEN v.in_stock THEN v.price_czk END), MIN(v.price_czk))
     FROM catalog_variants v WHERE v.catalog_product_id = catalog_products.id)
  SQL

  scope :active, -> { where(active: true) }
  scope :ordered, -> { order(:position, :id) }
  # Product lists: the price comes with the product instead of one or two
  # queries per product card.
  scope :with_starting_price, -> { select(arel_table[Arel.star], Arel.sql("#{STARTING_PRICE_SQL} AS starting_price_czk")) }

  def starting_price_czk
    return self[:starting_price_czk] if has_attribute?(:starting_price_czk)

    self.class.where(id: id).pick(Arel.sql(STARTING_PRICE_SQL))
  end
end
