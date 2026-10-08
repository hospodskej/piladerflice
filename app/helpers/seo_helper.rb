module SeoHelper
  # The German version is for Austria.
  HREFLANG = { "cs" => "cs", "de" => "de-AT" }.freeze

  # <link rel="canonical"> and the hreflang alternates for the Czech and
  # German versions of a public page (the pages page_meta gave a description).
  # Only the query parameters that change what the page shows are kept.
  def alternate_links
    return unless @indexable_page

    hrefs = language_urls

    safe_join([
      tag.link(rel: "canonical", href: hrefs.fetch(I18n.locale.to_s)),
      *hrefs.map { |locale, href| tag.link(rel: "alternate", hreflang: HREFLANG.fetch(locale), href: href) },
      tag.link(rel: "alternate", hreflang: "x-default", href: hrefs.fetch("cs"))
    ], "\n")
  end

  # Open Graph + Twitter tags: the preview card shown when a link is shared in
  # WhatsApp, Facebook, iMessage, Slack... Uses the page's own title and
  # description, and the share image of the current language when there is one
  # (see config/initializers/share_images.rb).
  OG_LOCALES = { "cs" => "cs_CZ", "de" => "de_AT" }.freeze
  IMAGE_TYPES = { ".webp" => "image/webp", ".png" => "image/png", ".jpg" => "image/jpeg", ".jpeg" => "image/jpeg" }.freeze

  def open_graph_tags
    locale = I18n.locale.to_s
    image = Rails.application.config.x.share_images[locale]
    tags = [
      og("og:type", "website"),
      og("og:site_name", t("site.name")),
      og("og:title", content_for(:title).presence || t("site.name")),
      og("og:url", @indexable_page ? language_urls.fetch(locale) : request.base_url + request.path),
      og("og:locale", OG_LOCALES.fetch(locale)),
      *(OG_LOCALES.except(locale).values.map { |other| og("og:locale:alternate", other) })
    ]
    tags << og("og:description", content_for(:description)) if content_for?(:description)
    if image
      tags.push(og("og:image", image_url(image)), og("og:image:type", IMAGE_TYPES.fetch(File.extname(image).downcase)),
                og("og:image:width", 1200), og("og:image:height", 630), og("og:image:alt", t("site.name")))
    end
    tags << tag.meta(name: "twitter:card", content: image ? "summary_large_image" : "summary")

    safe_join(tags, "\n")
  end

  # schema.org LocalBusiness data for search engines (JSON-LD). The facts are
  # the same ones shown in the footer and on the contact page; keep them in
  # sync. Deliberately no aggregateRating: Google doesn't accept a business's
  # own self-published ratings, and the Google reviews are already on Google.
  def local_business_schema
    {
      "@context" => "https://schema.org",
      "@type" => "LocalBusiness",
      "@id" => "#{root_url}#business",
      "name" => t("site.name"),
      "legalName" => "Štěpán Merta",
      "description" => t("meta.home.description"),
      "url" => root_url,
      "logo" => image_url("logo-wm.webp"),
      "image" => image_url("logo-wm.webp"),
      "telephone" => ["+420602446339", "+420515235527"],
      "email" => "stepan.merta@seznam.cz",
      "vatID" => "CZ6909094742",
      "taxID" => "45665451",
      "address" => {
        "@type" => "PostalAddress",
        "streetAddress" => "Derflice 66",
        "postalCode" => "671 28",
        "addressLocality" => "Znojmo",
        "addressCountry" => "CZ"
      },
      "geo" => { "@type" => "GeoCoordinates", "latitude" => 48.8096, "longitude" => 16.1211 },
      "openingHoursSpecification" => [{
        "@type" => "OpeningHoursSpecification",
        "dayOfWeek" => %w[Monday Tuesday Wednesday Thursday Friday],
        "opens" => "07:00",
        "closes" => "15:30"
      }],
      "areaServed" => [t("meta.area_served.south_moravia"), t("meta.area_served.north_austria")],
      "sameAs" => ["https://www.facebook.com/piladerflice/"]
    }
  end

  # <script type="application/ld+json"> with everything that could end the
  # script tag early (<, >, &) escaped.
  def json_ld_tag(data)
    tag.script(raw(json_escape(data.to_json)), type: "application/ld+json")
  end

  private

  def og(property, content)
    tag.meta(property: property, content: content)
  end

  # The same page in each language, absolute. Only the query parameters that
  # change what the page shows are kept.
  def language_urls
    path = LocalizedPath.strip(request.path)
    query = request.query_parameters.slice("category")
    suffix = query.any? ? "?#{query.to_query}" : ""
    %w[cs de].index_with { |locale| request.base_url + LocalizedPath.call(path, locale) + suffix }
  end
end
