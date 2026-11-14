# Site paths in a given language: the Austrian German site lives under /at,
# Czech (the default) has no prefix. The translations are German ("de"); the
# URL prefix names the market.
module LocalizedPath
  SEGMENTS = { "de" => "at" }.freeze # locale => URL prefix
  PREFIX = %r{\A/(?:#{SEGMENTS.values.join("|")})(?=[/?#]|\z)}

  # path may carry a query string and/or an #anchor.
  def self.call(path, locale)
    path = path.to_s
    return path unless path.start_with?("/")

    base = strip(path)
    segment = segment_for(locale)
    return base unless segment

    base = base.sub(%r{\A/(?=[?#]|\z)}, "")
    "/#{segment}#{base}"
  end

  # The same path without any language prefix.
  def self.strip(path)
    stripped = path.to_s.sub(PREFIX, "")
    stripped.start_with?("/") ? stripped : "/#{stripped}"
  end

  # "de" => "at"; nil for the unprefixed default language.
  def self.segment_for(locale)
    SEGMENTS[locale.to_s]
  end

  # "at" => "de"; nil when the segment isn't a language prefix.
  def self.locale_for(segment)
    SEGMENTS.key(segment.to_s)
  end
end
