# This file should ensure the existence of records required to run the application in every environment (production,
# development, test). The code here should be idempotent so that it can be executed at any point in every environment.
# The data can then be loaded with the bin/rails db:seed command (or created alongside the database with db:setup).
#
# Example:
#
#   ["Action", "Comedy", "Drama", "Horror"].each do |genre_name|
#     MovieGenre.find_or_create_by!(name: genre_name)
#   end
puts "Clearing old records..."
Product.destroy_all

puts "Creating Sortiment Products..."
Product.create([
                 { title: "Palivové dřevo", text: "Velikosti a druh podle volby zákazníka", image: "sortiment/palivove-drevo.png", link: "/sortiment/palivove-drevo" },
                 { title: "Stavební řezivo", text: "Hranoly, fošny, střešní latě, prkna", image: "sortiment/stavebni-rezivo.png", link: "/sortiment/stavebni-rezivo" },
                 { title: "Truhlářské řezivo", text: "Prkna, fošny", image: "sortiment/truhlarske-rezivo.png", link: "/sortiment/truhlarske-rezivo" },
                 { title: "Výrobní zbytky", text: "Piliny, hobliny, odřezky, štěpka, prokladky", image: "sortiment/vyrobni-zbytky.png", link: "/sortiment/vyrobni-zbytky" },
                 { title: "Okrasné kamenivo", text: "Kamenivo stavební frakce", image: "sortiment/okrasne-kamenivo.png", link: "/sortiment/okrasne-kamenivo" }
               ])

puts "Success! Created #{Product.count} products."

puts "Clearing old price list..."
PricelistItem.destroy_all

puts "Creating new price list..."

# 1. Stavební a truhlářské řezivo
PricelistItem.create!([
                        { category: "stavebni", item_name: "Fošny", details: "4m, 5m", price: "9 800 Kč / m<sup>3</sup>" },
                        { category: "stavebni", item_name: "Prkna (24mm)", details: "4m, 5m", price: "7 000 Kč / m<sup>3</sup>" },
                        { category: "stavebni", item_name: "Střešní latě", details: "4m, 5m", price: "9 800 Kč / m<sup>3</sup>" },
                        { category: "stavebni", item_name: "Trámy", details: "4m, 5m", price: "9 800 Kč / m<sup>3</sup>" }
                      ])

# 2. Palivové dřevo - volně ložené
PricelistItem.create!([
                        # Jasan / Dub
                        { category: "palivove_volne", subcategory: "Jasan / Dub", item_name: "1m", price: "1 680 Kč" },
                        { category: "palivove_volne", subcategory: "Jasan / Dub", item_name: "50cm", price: "1 980 Kč" },
                        { category: "palivove_volne", subcategory: "Jasan / Dub", item_name: "33cm", price: "1 680 Kč" },
                        { category: "palivove_volne", subcategory: "Jasan / Dub", item_name: "25cm", price: "1 780 Kč" },

                        # Buk / Akát
                        { category: "palivove_volne", subcategory: "Buk / Akát", item_name: "1m", price: "1 850 Kč" },
                        { category: "palivove_volne", subcategory: "Buk / Akát", item_name: "50cm", price: "2 150 Kč" },
                        { category: "palivove_volne", subcategory: "Buk / Akát", item_name: "33cm", price: "1 850 Kč" },
                        { category: "palivove_volne", subcategory: "Buk / Akát", item_name: "25cm", price: "1 950 Kč" },

                        # Bříza / Ostatní tvrdé
                        { category: "palivove_volne", subcategory: "Bříza / Ostatní tvrdé", item_name: "1m", price: "1 500 Kč" },
                        { category: "palivove_volne", subcategory: "Bříza / Ostatní tvrdé", item_name: "50cm", price: "1 800 Kč" },
                        { category: "palivove_volne", subcategory: "Bříza / Ostatní tvrdé", item_name: "33cm", price: "1 500 Kč" },
                        { category: "palivove_volne", subcategory: "Bříza / Ostatní tvrdé", item_name: "25cm", price: "1 600 Kč" },

                        # Smrk / Borovice / Modřín
                        { category: "palivove_volne", subcategory: "Smrk / Borovice / Modřín", item_name: "1m", price: "1 300 Kč" },
                        { category: "palivove_volne", subcategory: "Smrk / Borovice / Modřín", item_name: "50cm", price: "1 450 Kč" },
                        { category: "palivove_volne", subcategory: "Smrk / Borovice / Modřín", item_name: "33cm", price: "1 200 Kč" },
                        { category: "palivove_volne", subcategory: "Smrk / Borovice / Modřín", item_name: "25cm", price: "1 300 Kč" }
                      ])

# 3. Palivové dřevo - Bedny skládané
PricelistItem.create!([
                        # Jasan / Dub
                        { category: "palivove_skladane", subcategory: "Jasan / Dub", item_name: "50cm", price: "2 450 Kč" },
                        { category: "palivove_skladane", subcategory: "Jasan / Dub", item_name: "33cm", price: "2 450 Kč" },
                        { category: "palivove_skladane", subcategory: "Jasan / Dub", item_name: "25cm", price: "2 550 Kč" },

                        # Buk / Akát
                        { category: "palivove_skladane", subcategory: "Buk / Akát", item_name: "50cm", price: "2 600 Kč" },
                        { category: "palivove_skladane", subcategory: "Buk / Akát", item_name: "33cm", price: "2 600 Kč" },
                        { category: "palivove_skladane", subcategory: "Buk / Akát", item_name: "25cm", price: "2 700 Kč" },

                        # Smrk
                        { category: "palivove_skladane", subcategory: "Smrk", item_name: "50cm", price: "2 050 Kč" },
                        { category: "palivove_skladane", subcategory: "Smrk", item_name: "33cm", price: "2 050 Kč" },
                        { category: "palivove_skladane", subcategory: "Smrk", item_name: "25cm", price: "2 150 Kč" }
                      ])

# 4. Výrobní zbytky
PricelistItem.create!([
                        { category: "zbytky", item_name: "Odkory na topení", price: "1 000 Kč – 1 400 Kč" },
                        { category: "zbytky", item_name: "Piliny", price: "450 Kč" },
                        { category: "zbytky", item_name: "Štěpka", price: "700 - 800 Kč" }
                      ])

# 5. Okrasné kamenivo
PricelistItem.create!(category: "kamenivo", item_name: "Kamenivo", price: "1 Kč / 1 kg")

# 6. Služby
PricelistItem.create!([
                        { category: "sluzby", item_name: "Doprava", price: "35 Kč / km" },
                        { category: "sluzby", item_name: "Impregnace", price: "500 Kč" },
                        { category: "sluzby", item_name: "Pořez / Prodej kulatiny", price: "Po domluvě" },
                        { category: "sluzby", item_name: "Hoblování", price: "1 strana = 750 Kč / m<sup>3</sup>" }
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
  link: "#"
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
                    button_path: "/kontakt#kalkulace"
                  },
                  {
                    title: "Doprava",
                    content: "Na přání vám zpracované výrobky bezpečně doručíme až na místo určení. Rozvážíme palivové dřevo, kulatinu i řezivo – vše dopravíme ihned po výrobě, abyste měli materiál co nejdříve k dispozici. <br /><br />Doba výroby a následná doprava se odvíjí od rozsahu objednávky. O přesném termínu dodání vás vždy informujeme předem.",
                    images: ["sluzby/doprava1.png", "sluzby/doprava2.png"],
                    button_text: "Více o dopravě",
                    button_path: "/sluzby#doprava"
                  },
                  {
                    title: "Impregnace",
                    content: "Nabízíme možnost profesionální impregnace vašich dřevěných výrobků, a to pomocí moderní technologie v naší impregnační vaně. Impregnace výrazně prodlužuje životnost dřeva a chrání ho proti vlhkosti, škůdcům i houbám. <br /><br />Na přání zákazníka impregnujeme jakýkoli náš výrobek. Vše provádíme pečlivě a s důrazem na kvalitu výsledné ochrany.",
                    images: ["sluzby/impregnace1.png", "sluzby/impregnace2.png"],
                    button_text: "Více o impregnaci",
                    button_path: "/sluzby#impregnace"
                  },
                  {
                    title: "Hoblování",
                    content: "Nabízíme možnost ohoblování dřevěného materiálu přesně podle vašich požadavků. Disponujeme výkonnou hoblovačkou s hoblovacím průřezem až 80 cm, která si poradí i s rozměrnějšími kusy.<br><br>Hoblujeme nejčastěji fošny, prkna a hranoly libovolné délky. Zákazník si může určit, která strana bude pohledová, a hloubku úběru. Každý kus zpracováváme pečlivě, s důrazem na kvalitu povrchu a preciznost provedení.",
                    images: ["sluzby/hoblovani1.png", "sluzby/hoblovani2.png"],
                    button_text: "Více o hoblování",
                    button_path: "/sluzby#hoblovani"
                  },
                  {
                    title: "Pořez / Prodej kulatiny",
                    content: "Na základě vaší žádosti nabízíme možnost pořezání vaší vlastní kulatiny na míru podle požadovaných rozměrů. Pokud nemáte zájem o další zpracování, je možné kulatinu také jednoduše prodat přímo nám. <br /><br /> Tato služba je ideální pro soukromé vlastníky lesa, malé firmy nebo kohokoli, kdo chce efektivně využít vlastní dřevo bez starostí s technickým vybavením.",
                    images: ["sluzby/kulatina1.png", "sluzby/kulatina2.png"],
                    button_text: "Více o pořezu",
                    button_path: "/sluzby#porez-prodej-kulatiny"
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
                    image: "co-by-vas-mohlo-zajimat/nakladak.png"
                  },
                  {
                    title: "Jak správně skladovat dřevo?",
                    content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nullam in dui mauris.<br>Vivamus hendrerit arcu sed erat molestie vehicula. Sed auctor neque eu tellus rhoncus ut eleifend nibh porttitor.",
                    image: "co-by-vas-mohlo-zajimat/sklad.png"
                  },
                  {
                    title: "Kdy je nejlepší čas na nákup?",
                    content: "Praesent commodo cursus magna, vel scelerisque nisl consectetur et. Donec id elit non mi porta gravida at eget metus.<br>Aenean lacinia bibendum nulla sed consectetur.",
                    image: "co-by-vas-mohlo-zajimat/cas.png"
                  }
                ])

puts "Successfully created #{FaqItem.count} FAQ!"
