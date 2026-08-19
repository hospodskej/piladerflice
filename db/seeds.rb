# This file should ensure the existence of records required to run the application in every environment (production,
# development, test). The code here should be idempotent so that it can be executed at any point in every environment.
# The data can then be loaded with the bin/rails db:seed command (or created alongside the database with db:setup).
#
# NOTE ON TRANSLATIONS: every text column here has a matching "<column>_de"
# German counterpart (see the AddGermanTranslations migration and the
# Translatable concern in app/models/concerns/translatable.rb). Purely
# numeric/currency values (prices, dimensions) are left without a German
# override since there's nothing to translate - the *_i18n accessors will
# fall back to the Czech column automatically when *_de is blank.
puts "Clearing old records..."
Product.destroy_all

puts "Creating Sortiment Products..."
Product.create([
                 { title: "Palivové dřevo", text: "Velikosti a druh podle volby zákazníka", image: "sortiment/palivove-drevo.png", link: "/sortiment/palivove-drevo",
                   title_de: "Brennholz", text_de: "Größe und Holzart nach Kundenwunsch" },
                 { title: "Stavební řezivo", text: "Hranoly, fošny, střešní latě, prkna", image: "sortiment/stavebni-rezivo.png", link: "/sortiment/stavebni-rezivo",
                   title_de: "Bauschnittholz", text_de: "Kanthölzer, Bohlen, Dachlatten, Bretter" },
                 { title: "Truhlářské řezivo", text: "Prkna, fošny", image: "sortiment/truhlarske-rezivo.png", link: "/sortiment/truhlarske-rezivo",
                   title_de: "Tischlerholz", text_de: "Bretter, Bohlen" },
                 { title: "Výrobní zbytky", text: "Piliny, hobliny, odřezky, štěpka, prokladky", image: "sortiment/vyrobni-zbytky.png", link: "/sortiment/vyrobni-zbytky",
                   title_de: "Produktionsreste", text_de: "Sägemehl, Hobelspäne, Abschnitte, Hackschnitzel, Zwischenlagen" },
                 { title: "Okrasné kamenivo", text: "Kamenivo stavební frakce", image: "sortiment/okrasne-kamenivo.png", link: "/sortiment/okrasne-kamenivo",
                   title_de: "Ziersteine", text_de: "Bauschotter-Körnung" }
               ])

puts "Success! Created #{Product.count} products."

puts "Clearing old price list..."
PricelistItem.destroy_all

puts "Creating new price list..."

# 1. Stavební a truhlářské řezivo
PricelistItem.create!([
                        { category: "stavebni", item_name: "Fošny", details: "4m, 5m", price: "9 800 Kč / m<sup>3</sup>", item_name_de: "Bohlen" },
                        { category: "stavebni", item_name: "Prkna (24mm)", details: "4m, 5m", price: "7 000 Kč / m<sup>3</sup>", item_name_de: "Bretter (24mm)" },
                        { category: "stavebni", item_name: "Střešní latě", details: "4m, 5m", price: "9 800 Kč / m<sup>3</sup>", item_name_de: "Dachlatten" },
                        { category: "stavebni", item_name: "Trámy", details: "4m, 5m", price: "9 800 Kč / m<sup>3</sup>", item_name_de: "Balken" }
                      ])

# 2. Palivové dřevo - volně ložené
# Prices below are entered directly from the business's official printed
# price sheets (CZK sheet valid from 20.07.2026, matching EUR sheet valid
# from the same date) - both currencies are fixed values here, not
# calculated, unlike every other category (see PricelistItem#price_i18n).
# Subcategories mirror the original grouping (species priced identically
# share one row); "Smrk" on the new sheet applies to the whole
# "Smrk / Borovice / Modřín" group, and "Bříza / Ostatní tvrdé" isn't on
# the new sheet at all, so it shows "---".
PricelistItem.create!([
                        # Jasan / Dub
                        { category: "palivove_volne", subcategory: "Jasan / Dub", subcategory_de: "Esche / Eiche", item_name: "1m", price: "1 740 Kč", price_de: "67 €" },
                        { category: "palivove_volne", subcategory: "Jasan / Dub", subcategory_de: "Esche / Eiche", item_name: "50cm", price: "2 030 Kč", price_de: "78 €" },
                        { category: "palivove_volne", subcategory: "Jasan / Dub", subcategory_de: "Esche / Eiche", item_name: "33cm", price: "1 740 Kč", price_de: "67 €" },
                        { category: "palivove_volne", subcategory: "Jasan / Dub", subcategory_de: "Esche / Eiche", item_name: "25cm", price: "1 840 Kč", price_de: "69 €" },

                        # Buk / Akát
                        { category: "palivove_volne", subcategory: "Buk / Akát", subcategory_de: "Buche / Akazie", item_name: "1m", price: "1 900 Kč", price_de: "74 €" },
                        { category: "palivove_volne", subcategory: "Buk / Akát", subcategory_de: "Buche / Akazie", item_name: "50cm", price: "2 200 Kč", price_de: "84 €" },
                        { category: "palivove_volne", subcategory: "Buk / Akát", subcategory_de: "Buche / Akazie", item_name: "33cm", price: "1 900 Kč", price_de: "74 €" },
                        { category: "palivove_volne", subcategory: "Buk / Akát", subcategory_de: "Buche / Akazie", item_name: "25cm", price: "2 000 Kč", price_de: "72 €" },

                        # Bříza / Ostatní tvrdé - not on the current price sheet
                        { category: "palivove_volne", subcategory: "Bříza / Ostatní tvrdé", subcategory_de: "Birke / Sonstiges Hartholz", item_name: "1m", price: "---", price_de: "---" },
                        { category: "palivove_volne", subcategory: "Bříza / Ostatní tvrdé", subcategory_de: "Birke / Sonstiges Hartholz", item_name: "50cm", price: "---", price_de: "---" },
                        { category: "palivove_volne", subcategory: "Bříza / Ostatní tvrdé", subcategory_de: "Birke / Sonstiges Hartholz", item_name: "33cm", price: "---", price_de: "---" },
                        { category: "palivove_volne", subcategory: "Bříza / Ostatní tvrdé", subcategory_de: "Birke / Sonstiges Hartholz", item_name: "25cm", price: "---", price_de: "---" },

                        # Smrk / Borovice / Modřín
                        { category: "palivove_volne", subcategory: "Smrk / Borovice / Modřín", subcategory_de: "Fichte / Kiefer / Lärche", item_name: "1m", price: "1 320 Kč", price_de: "54 €" },
                        { category: "palivove_volne", subcategory: "Smrk / Borovice / Modřín", subcategory_de: "Fichte / Kiefer / Lärche", item_name: "50cm", price: "1 510 Kč", price_de: "58 €" },
                        { category: "palivove_volne", subcategory: "Smrk / Borovice / Modřín", subcategory_de: "Fichte / Kiefer / Lärche", item_name: "33cm", price: "1 260 Kč", price_de: "54 €" },
                        { category: "palivove_volne", subcategory: "Smrk / Borovice / Modřín", subcategory_de: "Fichte / Kiefer / Lärche", item_name: "25cm", price: "1 370 Kč", price_de: "54 €" }
                      ])

# 3. Palivové dřevo - Bedny skládané
PricelistItem.create!([
                        # Jasan / Dub
                        { category: "palivove_skladane", subcategory: "Jasan / Dub", subcategory_de: "Esche / Eiche", item_name: "50cm", price: "2 525 Kč", price_de: "97,30 €" },
                        { category: "palivove_skladane", subcategory: "Jasan / Dub", subcategory_de: "Esche / Eiche", item_name: "33cm", price: "2 525 Kč", price_de: "97,30 €" },
                        { category: "palivove_skladane", subcategory: "Jasan / Dub", subcategory_de: "Esche / Eiche", item_name: "25cm", price: "2 625 Kč", price_de: "102,20 €" },

                        # Buk / Akát
                        { category: "palivove_skladane", subcategory: "Buk / Akát", subcategory_de: "Buche / Akazie", item_name: "50cm", price: "2 675 Kč", price_de: "103,00 €" },
                        { category: "palivove_skladane", subcategory: "Buk / Akát", subcategory_de: "Buche / Akazie", item_name: "33cm", price: "2 675 Kč", price_de: "103,00 €" },
                        { category: "palivove_skladane", subcategory: "Buk / Akát", subcategory_de: "Buche / Akazie", item_name: "25cm", price: "2 775 Kč", price_de: "107,00 €" },

                        # Smrk
                        { category: "palivove_skladane", subcategory: "Smrk", subcategory_de: "Fichte", item_name: "50cm", price: "2 125 Kč", price_de: "81,00 €" },
                        { category: "palivove_skladane", subcategory: "Smrk", subcategory_de: "Fichte", item_name: "33cm", price: "2 125 Kč", price_de: "81,00 €" },
                        { category: "palivove_skladane", subcategory: "Smrk", subcategory_de: "Fichte", item_name: "25cm", price: "2 225 Kč", price_de: "85,60 €" }
                      ])

# 4. Výrobní zbytky
PricelistItem.create!([
                        { category: "zbytky", item_name: "Odkory na topení", price: "1 000 Kč – 1 400 Kč", item_name_de: "Rindenreste zum Heizen" },
                        { category: "zbytky", item_name: "Piliny", price: "450 Kč", item_name_de: "Sägemehl" },
                        { category: "zbytky", item_name: "Štěpka", price: "700 Kč – 800 Kč", item_name_de: "Hackschnitzel" }
                      ])

# 5. Okrasné kamenivo
PricelistItem.create!(category: "kamenivo", item_name: "Kamenivo", price: "1 Kč / 1 kg", item_name_de: "Ziersteine")

# 6. Služby
PricelistItem.create!([
                        { category: "sluzby", item_name: "Doprava", price: "35 Kč / km", item_name_de: "Lieferung" },
                        { category: "sluzby", item_name: "Impregnace", price: "500 Kč", item_name_de: "Imprägnierung" },
                        { category: "sluzby", item_name: "Pořez / Prodej kulatiny", price: "Po domluvě", item_name_de: "Sägen / Verkauf von Rundholz", price_de: "Nach Vereinbarung" },
                        { category: "sluzby", item_name: "Hoblování", price: "1 strana = 750 Kč / m<sup>3</sup>", item_name_de: "Hobeln", price_de: "1 Seite = 750 Kč / m<sup>3</sup>" }
                      ])

puts "Price list was successfully imported!"

puts "Clearing old promos..."
Promo.destroy_all

puts "Creating Hero Promo..."
Promo.create!(
  title: "PALIVOVÉ DŘEVO – AKÁT 32 CM",
  feature_1: "Vysoká výhřevnost",
  feature_2: "Skladem ihned",
  feature_3: "Rychlé dodání",
  price: "od 1500 / 1 PRMS",
  image: "akcni-nabidka-hero.png",
  link: "/eshop/akat",
  title_de: "BRENNHOLZ – AKAZIE 32 CM",
  feature_1_de: "Hoher Heizwert",
  feature_2_de: "Sofort auf Lager",
  feature_3_de: "Schnelle Lieferung",
  price_de: "ab 1500 / 1 Rm"
)

puts "Promo successfully created!"

puts "Clearing old services..."
Service.destroy_all

puts "Creating services..."
Service.create!([
                  {
                    title: "Cenová kalkulace",
                    content: "Nabízíme nezávaznou cenovou kalkulaci našich výrobků a služeb, která vám pomůže lépe se rozhodnout před nákupem. <br /><br /> Díky přesné ceně předem víte, co vás čeká – bez závazků a bez překvapení.",
                    images: [],
                    button_text: "Nezávazná kalkulace",
                    button_path: "/kontakt#kalkulace",
                    title_de: "Preiskalkulation",
                    content_de: "Wir bieten eine unverbindliche Preiskalkulation unserer Produkte und Dienstleistungen an, die Ihnen die Kaufentscheidung erleichtert. <br /><br /> Dank des genauen Preises im Voraus wissen Sie, was Sie erwartet – ohne Verpflichtungen und ohne Überraschungen.",
                    button_text_de: "Unverbindliche Kalkulation"
                  },
                  {
                    title: "Doprava",
                    content: "Na přání vám zpracované výrobky bezpečně doručíme až na místo určení. Rozvážíme palivové dřevo, kulatinu i řezivo – vše dopravíme ihned po výrobě, abyste měli materiál co nejdříve k dispozici. <br /><br />Doba výroby a následná doprava se odvíjí od rozsahu objednávky. O přesném termínu dodání vás vždy informujeme předem.",
                    images: ["sluzby/doprava1.png", "sluzby/doprava2.png"],
                    button_text: "Více o dopravě",
                    button_path: "/sluzby#doprava",
                    title_de: "Lieferung",
                    content_de: "Auf Wunsch liefern wir die fertigen Produkte sicher direkt an Ihren Bestimmungsort. Wir liefern Brennholz, Rundholz und Schnittholz – alles wird sofort nach der Fertigung transportiert, damit Sie das Material so schnell wie möglich erhalten. <br /><br />Die Fertigungs- und anschließende Lieferzeit richtet sich nach dem Umfang der Bestellung. Über den genauen Liefertermin informieren wir Sie stets im Voraus.",
                    button_text_de: "Mehr über Lieferung"
                  },
                  {
                    title: "Impregnace",
                    content: "Nabízíme možnost profesionální impregnace vašich dřevěných výrobků, a to pomocí moderní technologie v naší impregnační vaně. Impregnace výrazně prodlužuje životnost dřeva a chrání ho proti vlhkosti, škůdcům i houbám. <br /><br />Na přání zákazníka impregnujeme jakýkoli náš výrobek. Vše provádíme pečlivě a s důrazem na kvalitu výsledné ochrany.",
                    images: ["sluzby/impregnace1.png", "sluzby/impregnace2.png"],
                    button_text: "Více o impregnaci",
                    button_path: "/sluzby#impregnace",
                    title_de: "Imprägnierung",
                    content_de: "Wir bieten die professionelle Imprägnierung Ihrer Holzprodukte mittels moderner Technologie in unserer Imprägnierwanne an. Die Imprägnierung verlängert die Lebensdauer des Holzes erheblich und schützt es vor Feuchtigkeit, Schädlingen und Pilzen. <br /><br />Auf Kundenwunsch imprägnieren wir jedes unserer Produkte. Wir arbeiten stets sorgfältig und mit Fokus auf die Qualität des Schutzes.",
                    button_text_de: "Mehr über Imprägnierung"
                  },
                  {
                    title: "Hoblování",
                    content: "Nabízíme možnost ohoblování dřevěného materiálu přesně podle vašich požadavků. Disponujeme výkonnou hoblovačkou s hoblovacím průřezem až 80 cm, která si poradí i s rozměrnějšími kusy.<br><br>Hoblujeme nejčastěji fošny, prkna a hranoly libovolné délky. Zákazník si může určit, která strana bude pohledová, a hloubku úběru. Každý kus zpracováváme pečlivě, s důrazem na kvalitu povrchu a preciznost provedení.",
                    images: ["sluzby/hoblovani1.png", "sluzby/hoblovani2.png"],
                    button_text: "Více o hoblování",
                    button_path: "/sluzby#hoblovani",
                    title_de: "Hobeln",
                    content_de: "Wir bieten das Hobeln von Holzmaterial genau nach Ihren Vorgaben an. Wir verfügen über eine leistungsstarke Hobelmaschine mit einem Hobelquerschnitt bis 80 cm, die auch größere Werkstücke bewältigt.<br><br>Am häufigsten hobeln wir Bohlen, Bretter und Kanthölzer beliebiger Länge. Der Kunde kann bestimmen, welche Seite die Sichtseite sein soll, sowie die Abtragstiefe. Jedes Stück bearbeiten wir sorgfältig, mit Fokus auf Oberflächenqualität und Präzision.",
                    button_text_de: "Mehr über Hobeln"
                  },
                  {
                    title: "Pořez / Prodej kulatiny",
                    content: "Na základě vaší žádosti nabízíme možnost pořezání vaší vlastní kulatiny na míru podle požadovaných rozměrů. Pokud nemáte zájem o další zpracování, je možné kulatinu také jednoduše prodat přímo nám. <br /><br /> Tato služba je ideální pro soukromé vlastníky lesa, malé firmy nebo kohokoli, kdo chce efektivně využít vlastní dřevo bez starostí s technickým vybavením.",
                    images: ["sluzby/kulatina1.png", "sluzby/kulatina2.png"],
                    button_text: "Více o pořezu",
                    button_path: "/sluzby#porez-prodej-kulatiny",
                    title_de: "Sägen / Verkauf von Rundholz",
                    content_de: "Auf Ihren Wunsch bieten wir das Sägen Ihres eigenen Rundholzes nach Maß gemäß den gewünschten Abmessungen an. Wenn Sie an einer weiteren Verarbeitung nicht interessiert sind, können Sie das Rundholz auch einfach direkt an uns verkaufen. <br /><br /> Dieser Service ist ideal für private Waldbesitzer, kleine Unternehmen oder jeden, der sein eigenes Holz effizient nutzen möchte, ohne sich um technische Ausrüstung kümmern zu müssen.",
                    button_text_de: "Mehr über das Sägen"
                  }
                ])

puts "Successfully created #{Service.count} services!"

puts "Clearing FAQ..."
FaqItem.destroy_all

puts "Creatng FAQ..."
FaqItem.create!([
                  {
                    title: "Kolik stojí doprava?",
                    content: "Cena dopravy se odvíjí od vzdálenosti a množství objednaného dřeva.<br>Po předání adresy a požadovaného množství vám vždy spočítáme přesnou cenu, aby byla pro vás co nejvýhodnější.",
                    image: "co-by-vas-mohlo-zajimat/nakladak.png",
                    title_de: "Was kostet die Lieferung?",
                    content_de: "Der Lieferpreis richtet sich nach Entfernung und bestellter Holzmenge.<br>Nach Angabe der Adresse und der gewünschten Menge berechnen wir Ihnen stets den genauen Preis, damit er für Sie möglichst günstig ist."
                  },
                  {
                    title: "Jak správně skladovat dřevo?",
                    content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nullam in dui mauris.<br>Vivamus hendrerit arcu sed erat molestie vehicula. Sed auctor neque eu tellus rhoncus ut eleifend nibh porttitor.",
                    image: "co-by-vas-mohlo-zajimat/sklad.png",
                    title_de: "Wie lagert man Holz richtig?",
                    content_de: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nullam in dui mauris.<br>Vivamus hendrerit arcu sed erat molestie vehicula. Sed auctor neque eu tellus rhoncus ut eleifend nibh porttitor."
                  },
                  {
                    title: "Kdy je nejlepší čas na nákup?",
                    content: "Praesent commodo cursus magna, vel scelerisque nisl consectetur et. Donec id elit non mi porta gravida at eget metus.<br>Aenean lacinia bibendum nulla sed consectetur.",
                    image: "co-by-vas-mohlo-zajimat/cas.png",
                    title_de: "Wann ist die beste Zeit zum Kauf?",
                    content_de: "Praesent commodo cursus magna, vel scelerisque nisl consectetur et. Donec id elit non mi porta gravida at eget metus.<br>Aenean lacinia bibendum nulla sed consectetur."
                  }
                ])

puts "Successfully created #{FaqItem.count} FAQ!"
