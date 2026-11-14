# Addresses of the old WordPress site (www.piladerflice.cz, until the 2026
# launch), taken from its sitemaps. Google and people's bookmarks still know
# them, so each one sends a permanent (301) redirect to the page that replaced
# it instead of a 404. Addresses the new site kept (/kontakt, /sluzby,
# /sortiment/..., /kosik, /muj-ucet) need nothing here.

# www.piladerflice.cz → piladerflice.cz, same path, when APP_HOST is set
# (production). The old site lived on www, so this catches all its traffic.
if (canonical_host = ENV["APP_HOST"].presence)
  constraints(->(request) { request.host == "www.#{canonical_host}" }) do
    match "/", to: redirect { |_params, request| "https://#{canonical_host}#{request.fullpath}" }, via: :all
    match "*path", to: redirect { |_params, request| "https://#{canonical_host}#{request.fullpath}" }, via: :all, format: false
  end
end

{
  "/obchod" => "/eshop",
  "/obchod/palivove-drevo" => "/eshop?category=palivove",
  "/obchod/stavebni-rezivo" => "/eshop?category=rezivo",
  "/obchod/vyrobni-zbytky" => "/eshop?category=zbytky",
  "/obchod/okrasne-kamenivo" => "/eshop?category=kamenivo",
  "/pokladna" => "/kosik",
  "/zakazkova-vyroba" => "/kontakt#kalkulace",
  "/fotogalerie" => "/sortiment",
  "/zasady-cookies-eu" => "/zasady-cookies",

  "/produkt/palivove-drevo-mekke-smrk" => "/eshop/smrk",
  "/produkt/palivove-drevo-tvrde-akat" => "/eshop/akat",
  "/produkt/palivove-drevo-tvrde-briza" => "/eshop/briza",
  "/produkt/palivove-drevo-tvrde-buk" => "/eshop/buk",
  "/produkt/palivove-drevo-tvrde-dub" => "/eshop/dub",
  "/produkt/palivove-drevo-tvrde-habr" => "/eshop/habr",
  "/produkt/palivove-drevo-tvrde-jasan" => "/eshop/jasan",
  "/produkt/hranoly" => "/eshop/tramy",
  "/produkt/fosny" => "/eshop/fosny",
  "/produkt/prkna-24-mm" => "/eshop/prkna",
  "/produkt/stresni-late" => "/eshop/late",
  "/produkt/odkory-na-topeni" => "/eshop/odkory",
  "/produkt/piliny" => "/eshop/piliny",
  "/produkt/stepka" => "/eshop/stepka",
  "/produkt/okrasne-kamenivo" => "/eshop/okrasne_kamenivo",

  # The German site was under /de, it's /at now.
  "/de" => "/at",
  "/de/sortiment" => "/at/sortiment",
  "/de/sortiment/brennholz" => "/at/sortiment/palivove-drevo",
  "/de/sortiment/bauholz" => "/at/sortiment/stavebni-rezivo",
  "/de/sortiment/tischlerholz" => "/at/sortiment/truhlarske-rezivo",
  "/de/sortiment/produktionsreste" => "/at/sortiment/vyrobni-zbytky",
  "/de/sortiment/ornamentale-aggregate" => "/at/sortiment/okrasne-kamenivo",
  "/de/eshop/brennholz-eshop" => "/at/eshop?category=palivove",
  "/de/eshop/bauholz-eshop" => "/at/eshop?category=rezivo",
  "/de/eshop/produktionsreste-eshop" => "/at/eshop?category=zbytky",
  "/de/eshop/ziersteine-eshop" => "/at/eshop?category=kamenivo",
  "/de/massanfertigung" => "/at/kontakt#kalkulace",
  "/de/dienstleistungen" => "/at/sluzby",
  "/de/fotogalerie" => "/at/sortiment",
  "/de/kontakt" => "/at/kontakt",
  "/de/warenkorb" => "/at/kosik",
  "/de/zur-kasse" => "/at/kosik"
}.each do |old_path, new_path|
  get old_path, to: redirect(new_path)
end

# Anything else the old site had under these goes to the closest listing.
get "/produkt/*rest", to: redirect("/eshop"), format: false
get "/obchod/*rest", to: redirect("/eshop"), format: false
get "/de/*rest", to: redirect("/at"), format: false
