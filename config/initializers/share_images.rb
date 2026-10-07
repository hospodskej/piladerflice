# Social share cards (what WhatsApp, Facebook, iMessage... show when a link is
# pasted): one 1200x630 PNG or JPG per language, dropped into
# app/assets/images/share/ as share-cs.png and share-de.png. The Czech card is
# used for the plain URLs and the German one for /at. A language without a card
# still gets the title and description, just no picture.
# Restart the server after adding or replacing a file.
Rails.application.config.x.share_images = %w[cs de].each_with_object({}) do |locale, found|
  file = Dir[Rails.root.join("app/assets/images/share/share-#{locale}.{png,jpg,jpeg}")].min
  found[locale] = "share/#{File.basename(file)}" if file
end
