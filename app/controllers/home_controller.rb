class HomeController < ApplicationController
  BESTSELLER_KEYS = %w[akat tramy dub].freeze

  def index
    @hero_promo = Promo.weekly_pick
    @sluzby = Service.all
    @zajimavosti = FaqItem.all
    @produkty = Product.all
    @google_reviews = Review.ordered
    @bestseller_products = CatalogProduct.active.where(key: BESTSELLER_KEYS).index_by(&:key).values_at(*BESTSELLER_KEYS).compact
  end

  def kontakt
    pricelist = PricelistItem.ordered.group_by(&:category)
    @stavebni_data = pricelist.fetch("stavebni", [])
    @palivove_volne_groups = pricelist.fetch("palivove_volne", []).group_by(&:subcategory_i18n)
    @palivove_skladane_groups = pricelist.fetch("palivove_skladane", []).group_by(&:subcategory_i18n)
    @vyrobni_zbytky = pricelist.fetch("zbytky", [])
    @kamenivo_item = pricelist.fetch("kamenivo", []).first
    @sluzby_cenik = pricelist.fetch("sluzby", [])
    @kalkulace_options = InquiryFormOption.ordered.group_by(&:category).transform_values { |options| options.group_by(&:field) }
  end

  def sluzby
    @sluzby_page_data = Service.where("button_path LIKE ?", "/sluzby%")
  end

  def sortiment
    @kategorie = Product.all
  end

  def palivove_drevo
    render "home/sortiment/palivove_drevo"
  end

  def stavebni_rezivo
    render "home/sortiment/stavebni_rezivo"
  end

  def truhlarske_rezivo
    render "home/sortiment/truhlarske_rezivo"
  end

  def sortiment_okrasne_kamenivo
    render "home/sortiment/okrasne_kamenivo"
  end

  def vyrobni_zbytky
    render "home/sortiment/vyrobni_zbytky"
  end

  def eshop
    render template: "home/eshop"
  end

  def product
    @product = CatalogProduct.active.find_by!(key: params[:key])
  end
end
