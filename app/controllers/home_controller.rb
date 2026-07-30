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

  def palivove_drevo
    render "home/sortiment/palivove_drevo"
    @eshop_kroky = [
      "Vyber produkt v e-shopu",
      "Přidejte do košíku",
      "Zadejte kontaktní údaje a dopravu",
      "Přejděte k dokončení objednávky"
    ]
    @poptavka_kroky = [
      "Poptáte nám, co potřebujete",
      "Připravíme přesnou kalkulaci a návrh",
      "Na základě domluvy výrobek vyrobíme",
      "Doručíme až k Vám nebo připravíme k osobnímu odběru"
    ]
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

  def piliny
    render "home/eshop/piliny"
  end

  def vyrobni_zbytky
    render "home/sortiment/vyrobni_zbytky"
  end

  def eshop
    render template: "home/eshop"
  end

  def habr
    render "home/eshop/habr"
  end

  def briza
    render "home/eshop/briza"
  end

  def jasan
    render "home/eshop/jasan"
  end

  def akat
    render "home/eshop/akat"
  end

  def buk
    render "home/eshop/buk"
  end

  def dub
    render "home/eshop/dub"
  end

  def smrk
    render "home/eshop/smrk"
  end

  def late
    render "home/eshop/late"
  end

  def tramy
    render "home/eshop/tramy"
  end

  def fosny
    render "home/eshop/fosny"
  end

  def prkna
    render "home/eshop/prkna"
  end

  def stepka
    render "home/eshop/stepka"
  end

  def odkory
    render "home/eshop/odkory"
  end

  def eshop_okrasne_kamenivo
    render "home/eshop/okrasne_kamenivo"
  end

end
