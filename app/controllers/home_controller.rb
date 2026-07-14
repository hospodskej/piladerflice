class HomeController < ApplicationController
  def index
    @hero_promo = Promo.first
    @sluzby = Service.all
    @zajimavosti = FaqItem.all
    @produkty = Product.all
  end

  def kontakt
    @stavebni_data = PricelistItem.where(category: "stavebni")
    @palivove_volne_groups = PricelistItem.where(category: "palivove_volne").group_by(&:subcategory)
    @palivove_skladane_groups = PricelistItem.where(category: "palivove_skladane").group_by(&:subcategory)
    @vyrobni_zbytky = PricelistItem.where(category: "zbytky")
    @kamenivo_item = PricelistItem.find_by(category: "kamenivo")
    @sluzby_cenik = PricelistItem.where(category: "sluzby")
  end

  def sluzby
    @sluzby_page_data = Service.where.not(title: "Cenová kalkulace")
  end

  def sortiment
    @kategorie = Product.all

    @eshop_kroky = [
      " Vyber produkt v e-shopu",
      " Přidejte do košíku",
      " Zadejte kontaktní údaje a dopravu",
      " Přejděte k dokončení objednávky"
    ]

    @poptavka_kroky = [
      " Poptáte nám, co potřebujete",
      " Připravíme přesnou kalkulaci a návrh",
      " Na základě domluvy výrobek vyrobíme",
      " Doručíme až k vám nebo připravíme k osobnímu odběru"
    ]
  end
end
