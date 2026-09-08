# Populates the admin-editable product catalog (CatalogProduct /
# CatalogVariant) from the content that used to be hardcoded directly in
# the eshop view templates. Run via `bin/rails db:seed` (loaded by
# db/seeds.rb) or standalone via `bin/rails runner db/seeds_catalog.rb`.
#
# IMPORTANT PRICING NOTE: every price below is carried over unchanged from
# what was already in the templates - and almost all of it was already
# placeholder/example data before this migration (the firewood
# container/crate prices in particular were a repeating dummy pattern:
# 21 500 / 17 863 / 21 500 Kč for literally every species and size). This
# script does not fix that - it only moves the exact same numbers into the
# database so an admin can edit them going forward. Real prices still need
# to be entered by hand through the admin panel.

puts "Clearing old catalog data..."
CatalogVariant.destroy_all
CatalogProduct.destroy_all

def add_firewood_variants(product, kontejner_price:, bedny_price:)
  lengths = ["100 cm", "50 cm", "33 cm", "25 cm"]
  amounts = [5, 10, 15]

  position = 0
  amounts.each do |amount|
    lengths.each do |len|
      product.catalog_variants.create!(
        key: "kontejner-#{len.parameterize}-#{amount}prm",
        variant_group: "kontejner",
        length_label: len,
        amount_value: amount,
        price_czk: kontejner_price,
        position: (position += 1)
      )
    end
  end

  ["50 cm", "33 cm", "25 cm"].each do |len|
    product.catalog_variants.create!(
      key: "bedny-#{len.parameterize}",
      variant_group: "bedny",
      length_label: len,
      amount_value: 1,
      price_czk: bedny_price,
      position: (position += 1)
    )
  end
end

puts "Creating firewood products (smrk, dub, buk, akat, jasan, briza, habr)..."

firewood = [
  { key: "smrk", hardness: "soft", title: "Smrk", title_de: "Fichte", type_label: "Měkké palivové dřevo", type_label_de: "Weiches Brennholz",
    subtitle: "Měkké palivové dřevo", subtitle_de: "Weiches Brennholz",
    description: "Lehké smrkové dřevo ideální pro rychlé vytopení krbů a kamen, snadno chytá plamen. Výhřevnost cca 1500–1700 kWh/m³, hoří 1–2 hodiny.",
    description_de: "Leichtes Fichtenholz, ideal für schnelles Aufheizen von Kaminen und Öfen, fängt leicht Feuer. Heizwert ca. 1500–1700 kWh/m³, brennt 1–2 Stunden.",
    drying_note: "Doporučeno sušit 6–12 měsíců na suchém, větraném místě.", drying_note_de: "Empfohlene Trocknungszeit 6–12 Monate an einem trockenen, gut belüfteten Ort.",
    image_alt: "Smrk palivové dřevo", image_alt_de: "Fichte Brennholz" },
  { key: "dub", hardness: "hard", title: "Dub", title_de: "Eiche", type_label: "Tvrdé palivové dřevo", type_label_de: "Hartes Brennholz",
    subtitle: "Tvrdé palivové dřevo", subtitle_de: "Hartes Brennholz",
    description: "Dubové dřevo s vysokou výhřevností a dlouhou dobou hoření, ideální pro krby a kamna. Výhřevnost cca 2100–2300 kWh/m³, hoří 3–4 hodiny.",
    description_de: "Eichenholz mit hohem Heizwert und langer Brenndauer, ideal für Kamine und Öfen. Heizwert ca. 2100–2300 kWh/m³, brennt 3–4 Stunden.",
    drying_note: "Doporučeno sušit 1–2 roky na dobře větraném místě.", drying_note_de: "Empfohlene Trocknungszeit 1–2 Jahre an einem gut belüfteten Ort.",
    image_alt: "Dub palivové dřevo", image_alt_de: "Eiche Brennholz" },
  { key: "buk", hardness: "hard", title: "Buk", title_de: "Buche", type_label: "Tvrdé palivové dřevo", type_label_de: "Hartes Brennholz",
    subtitle: "Tvrdé palivové dřevo", subtitle_de: "Hartes Brennholz",
    description: "Bukové dřevo s vysokou výhřevností a rovnoměrným hořením, ideální pro dlouhodobé vytápění. Výhřevnost cca 2100–2200 kWh/m³, hoří 2–3 hodiny.",
    description_de: "Buchenholz mit hohem Heizwert und gleichmäßigem Abbrand, ideal für langanhaltendes Heizen. Heizwert ca. 2100–2200 kWh/m³, brennt 2–3 Stunden.",
    drying_note: "Doporučeno sušit 1–2 roky.", drying_note_de: "Empfohlene Trocknungszeit 1–2 Jahre.",
    image_alt: "Buk palivové dřevo", image_alt_de: "Buche Brennholz" },
  { key: "akat", hardness: "hard", title: "Akát", title_de: "Akazie", type_label: "Tvrdé palivové dřevo", type_label_de: "Hartes Brennholz",
    subtitle: "Tvrdé palivové dřevo", subtitle_de: "Hartes Brennholz",
    description: "Břízové dřevo je oblíbené pro rovnoměrné hoření, minimální kouřivost a jemnou vůni. Skvěle se hodí pro vytápění v krbech a kamnech. Výhřevnost cca 2200–2500 kWh/m³, hoří 3–4 hodiny.",
    description_de: "Beliebt für gleichmäßigen Abbrand, minimale Rauchentwicklung und einen feinen Duft. Bestens geeignet zum Heizen in Kaminen und Öfen. Heizwert ca. 2200–2500 kWh/m³, brennt 3–4 Stunden.",
    drying_note: "Doporučeno sušit 1–2 roky.", drying_note_de: "Empfohlene Trocknungszeit 1–2 Jahre.",
    image_alt: "Akát palivové dřevo", image_alt_de: "Akazie Brennholz" },
  { key: "jasan", hardness: "hard", title: "Jasan", title_de: "Esche", type_label: "Tvrdé palivové dřevo", type_label_de: "Hartes Brennholz",
    subtitle: "Tvrdé palivové dřevo", subtitle_de: "Hartes Brennholz",
    description: "Jasanové dřevo poskytuje stabilní hoření a intenzivní teplo, vhodné pro krby i kamna. Výhřevnost cca 1900–2100 kWh/m³, hoří 2–3 hodiny.",
    description_de: "Eschenholz sorgt für einen stabilen Abbrand und intensive Wärme, geeignet für Kamine und Öfen. Heizwert ca. 1900–2100 kWh/m³, brennt 2–3 Stunden.",
    drying_note: "Doporučeno sušit 1–2 roky.", drying_note_de: "Empfohlene Trocknungszeit 1–2 Jahre.",
    image_alt: "Jasan palivové dřevo", image_alt_de: "Esche Brennholz" },
  { key: "briza", hardness: "hard", title: "Bříza", title_de: "Birke", type_label: "Tvrdé palivové dřevo", type_label_de: "Hartes Brennholz",
    subtitle: "Tvrdé palivové dřevo", subtitle_de: "Hartes Brennholz",
    description: "Břízové dřevo je oblíbené pro rovnoměrné hoření, minimální kouřivost a jemnou vůni. Skvěle se hodí pro vytápění v krbech a kamnech. Výhřevnost cca 1900–2100 kWh/m³, hoří 2–3 hodiny.",
    description_de: "Beliebt für gleichmäßigen Abbrand, minimale Rauchentwicklung und einen feinen Duft. Bestens geeignet zum Heizen in Kaminen und Öfen. Heizwert ca. 1900–2100 kWh/m³, brennt 2–3 Stunden.",
    drying_note: "Doporučeno sušit 1–2 roky.", drying_note_de: "Empfohlene Trocknungszeit 1–2 Jahre.",
    image_alt: "Bříza palivové dřevo", image_alt_de: "Birke Brennholz" },
  { key: "habr", hardness: "hard", title: "Habr", title_de: "Hainbuche", type_label: "Tvrdé palivové dřevo", type_label_de: "Hartes Brennholz",
    subtitle: "Tvrdé palivové dřevo", subtitle_de: "Hartes Brennholz",
    description: "Habr patří mezi nejvýhřevnější dřeva, poskytuje dlouhotrvající a intenzivní teplo. Výhřevnost cca 2300–2500 kWh/m³, hoří 3–4 hodiny.",
    description_de: "Hainbuche gehört zu den heizwertstärksten Hölzern und liefert lang anhaltende, intensive Wärme. Heizwert ca. 2300–2500 kWh/m³, brennt 3–4 Stunden.",
    drying_note: "Doporučeno sušit 1–2 roky.", drying_note_de: "Empfohlene Trocknungszeit 1–2 Jahre.",
    image_alt: "Habr palivové dřevo", image_alt_de: "Hainbuche Brennholz" }
]

firewood.each_with_index do |data, i|
  product = CatalogProduct.create!(
    key: data[:key], template: "firewood", category: "palivove", hardness: data[:hardness],
    image: "eshop/#{data[:key]}.png", position: i,
    title: data[:title], title_de: data[:title_de],
    type_label: data[:type_label], type_label_de: data[:type_label_de],
    subtitle: data[:subtitle], subtitle_de: data[:subtitle_de],
    description: data[:description], description_de: data[:description_de],
    drying_note: data[:drying_note], drying_note_de: data[:drying_note_de],
    image_alt: data[:image_alt], image_alt_de: data[:image_alt_de]
  )
  add_firewood_variants(product, kontejner_price: 21_500, bedny_price: 21_500)
end

puts "Creating lumber products (tramy, late, fosny, prkna)..."

tramy = CatalogProduct.create!(
  key: "tramy", template: "lumber", category: "rezivo", image: "eshop/tramy.png", position: 0,
  title: "Trámy", title_de: "Balken", type_label: "Sypané", type_label_de: "Lose",
  description: "Masivní dřevěné trámy určené pro konstrukce krovů, stropů a nosných částí staveb. Vyrobené z kvalitního dřeva s dlouhou životností.",
  description_de: "Massive Holzbalken für Dachstühle, Decken und tragende Bauteile. Aus hochwertigem Holz mit langer Lebensdauer gefertigt."
)
[["100/100", "3 000 mm"], ["100/100", "4 000 mm"], ["100/100", "5 000 mm"],
 ["100/120", "3 000 mm"], ["100/120", "4 000 mm"], ["100/120", "5 000 mm"],
 ["100/140", "3 000 mm"], ["100/140", "4 000 mm"], ["100/140", "5 000 mm"],
 ["100/160", "3 000 mm"], ["100/160", "4 000 mm"], ["100/160", "5 000 mm"]].each_with_index do |(dim, len), i|
  tramy.catalog_variants.create!(key: "#{dim.parameterize}-#{len.parameterize}", variant_label: "#{dim} mm", length_label: len, price_czk: 8_500, position: i)
end

late = CatalogProduct.create!(
  key: "late", template: "lumber", category: "rezivo", image: "eshop/late.png", position: 1,
  title: "Střešní latě", title_de: "Dachlatten", type_label: "Sypané", type_label_de: "Lose",
  description: "Stavební latě vhodné pro střešní konstrukce, rošty a další stavební aplikace. Vyrobené z kvalitního dřeva, dostupné v různých délkách a průřezech. Ideální pro přesné a spolehlivé konstrukce.",
  description_de: "Bauholzlatten geeignet für Dachkonstruktionen, Roste und weitere Bauanwendungen. Aus hochwertigem Holz, erhältlich in verschiedenen Längen und Querschnitten. Ideal für präzise und zuverlässige Konstruktionen."
)
[["40/50", "3 000 mm"], ["40/50", "4 000 mm"], ["40/50", "5 000 mm"],
 ["50/30", "3 000 mm"], ["50/30", "4 000 mm"], ["50/30", "5 000 mm"],
 ["60/40", "3 000 mm"], ["60/40", "4 000 mm"], ["60/40", "5 000 mm"]].each_with_index do |(dim, len), i|
  late.catalog_variants.create!(key: "#{dim.parameterize}-#{len.parameterize}", variant_label: "#{dim} mm", length_label: len, price_czk: 8_500, position: i)
end

fosny = CatalogProduct.create!(
  key: "fosny", template: "lumber", category: "rezivo", image: "eshop/fosny.png", position: 2,
  title: "Fošny", title_de: "Bohlen", type_label: "Sypané", type_label_de: "Lose",
  description: "Robustní fošny ideální pro výrobu podlah, bednění nebo konstrukční prvky. Dostupné v široké škále rozměrů a tlouštěk. Kvalitní dřevo zajišťuje pevnost a odolnost.",
  description_de: "Robuste Bohlen ideal für die Herstellung von Böden, Schalungen oder Konstruktionselementen. Erhältlich in einer großen Auswahl an Maßen und Stärken. Hochwertiges Holz sorgt für Festigkeit und Beständigkeit."
)
dims = %w[120x40 140x40 180x40 200x40 220x40 120x50 140x50 180x50 200x50 220x50 240x50 120x60 140x60 180x60 200x60 220x60 240x60]
i = 0
dims.each do |dim|
  ["4 000 mm", "5 000 mm"].each do |len|
    fosny.catalog_variants.create!(key: "#{dim}-#{len.parameterize}", variant_label: "#{dim} mm", length_label: len, price_czk: 8_500, position: (i += 1))
  end
end

prkna = CatalogProduct.create!(
  key: "prkna", template: "lumber", category: "rezivo", image: "eshop/prkna.png", position: 3,
  title: "Prkna 24 mm", title_de: "Bretter 24 mm", type_label: "24 mm", type_label_de: "24 mm",
  description: "Kvalitní stavební prkna vhodná pro bednění, podlahy a další stavební využití.",
  description_de: "Hochwertige Bauschnittbretter geeignet für Schalungen, Böden und weitere Bauanwendungen."
)
[["I. netříděné", "I. unsortiert", 6_500], ["I. tříděné", "I. sortiert", 7_000], ["II. tříděné", "II. sortiert", 5_000]].each_with_index do |(cs, de, price), row|
  ["3 000 mm", "4 000 mm", "5 000 mm"].each_with_index do |len, col|
    prkna.catalog_variants.create!(key: "#{cs.parameterize}-#{len.parameterize}", variant_label: cs, variant_label_de: de, length_label: len, price_czk: price, position: row * 3 + col)
  end
end

puts "Creating residual/gravel products (odkory, piliny, stepka, okrasne_kamenivo)..."

odkory = CatalogProduct.create!(
  key: "odkory", template: "simple_variant", category: "zbytky", image: "eshop/odkory.png", position: 0,
  title: "Odkory na topení", title_de: "Rindenreste zum Heizen", type_label: "Odkory", type_label_de: "Rindenreste",
  description: "Levné a efektivní palivo z odkorněných kusů dřeva. Vhodné pro kamna, krby a kotle. Poskytuje vysokou výhřevnost při minimálních nákladech. Nabízíme volně ložené nebo balené dle potřeb zákazníka.",
  description_de: "Günstiger und effizienter Brennstoff aus entrindeten Holzstücken. Geeignet für Öfen, Kamine und Heizkessel. Bietet hohen Heizwert bei minimalen Kosten. Erhältlich lose oder verpackt nach Kundenwunsch."
)
odkory.catalog_variants.create!(key: "smrk", variant_label: "Smrk", variant_label_de: "Fichte", price_czk: 800, position: 0)
odkory.catalog_variants.create!(key: "dub", variant_label: "Dub", variant_label_de: "Eiche", price_czk: 1_400, position: 1)

piliny = CatalogProduct.create!(
  key: "piliny", template: "simple_variant", category: "zbytky", image: "eshop/piliny.png", position: 1,
  title: "Piliny", title_de: "Sägemehl", type_label: "Piliny", type_label_de: "Sägemehl",
  description: "Kvalitní dřevěné piliny vhodné pro podestýlku, mulčování nebo lisování briket. Nízká vlhkost a jemná struktura zaručují snadnou manipulaci. Nabízíme volně ložené nebo balené varianty.",
  description_de: "Hochwertiges Sägemehl geeignet als Einstreu, zum Mulchen oder zum Pressen von Briketts. Niedrige Feuchtigkeit und feine Struktur sorgen für einfache Handhabung. Erhältlich lose oder verpackt."
)
piliny.catalog_variants.create!(key: "volne-lozene", variant_label: "Volně ložené", variant_label_de: "Lose geschüttet", price_czk: 450, position: 0)
piliny.catalog_variants.create!(key: "balene", variant_label: "Balené", variant_label_de: "Verpackt", price_czk: 450, position: 1)

stepka = CatalogProduct.create!(
  key: "stepka", template: "simple_variant", category: "zbytky", image: "eshop/stepka.png", position: 2,
  title: "Štěpka", title_de: "Hackschnitzel", type_label: "Štěpka", type_label_de: "Hackschnitzel",
  description: "Dřevní štěpka ideální jako ekologické palivo nebo mulčovací materiál pro zahrady. Vyrobená z kvalitního dřeva, nízká vlhkost zajišťuje vysokou výhřevnost. Dostupná ve volně ložené či balené podobě.",
  description_de: "Holzhackschnitzel, ideal als umweltfreundlicher Brennstoff oder Mulchmaterial für den Garten. Aus hochwertigem Holz hergestellt, niedrige Feuchtigkeit sorgt für hohen Heizwert. Erhältlich lose oder verpackt."
)
stepka.catalog_variants.create!(key: "smrk", variant_label: "Smrk", variant_label_de: "Fichte", price_czk: 700, in_stock: true, position: 0)
stepka.catalog_variants.create!(key: "dub", variant_label: "Dub", variant_label_de: "Eiche", price_czk: 800, in_stock: true, position: 1)
stepka.catalog_variants.create!(key: "buk", variant_label: "Buk", variant_label_de: "Buche", price_czk: 800, in_stock: false, position: 2)

kamenivo = CatalogProduct.create!(
  key: "okrasne_kamenivo", template: "simple_variant", category: "kamenivo", image: "eshop/kamenivo.png", position: 3,
  title: "Okrasné kamenivo", title_de: "Ziersteine", type_label: "Kamenivo", type_label_de: "Kies",
  description: "Přírodní okrasné kamenivo pro úpravu zahrad, obsypy ploch a dekorativní účely. Dostupné ve třech frakcích.",
  description_de: "Natürliche Ziersteine für die Gartengestaltung, Flächenschüttungen und dekorative Zwecke. Erhältlich in drei Körnungen."
)
kamenivo.catalog_variants.create!(key: "32-64", variant_label: "32/64 mm", price_czk: 20, position: 0)
kamenivo.catalog_variants.create!(key: "64-120", variant_label: "64/120 mm", price_czk: 30, position: 1)
kamenivo.catalog_variants.create!(key: "16-32", variant_label: "16/32 mm", price_czk: 1, position: 2)

puts "Catalog seeded: #{CatalogProduct.count} products, #{CatalogVariant.count} variants."
