# Site paths in a given language: German pages live under /de, Czech (the
# default) has no prefix.
module LocalizedPath
  PREFIX = %r{\A/de(?=[/?#]|\z)}

  # path may carry a query string and/or an #anchor.
  def self.call(path, locale)
    path = path.to_s
    return path unless path.start_with?("/")

    base = strip(path)
    return base unless locale.to_s == "de"

    base = base.sub(%r{\A/(?=[?#]|\z)}, "")
    "/de#{base}"
  end

  # The same path without any language prefix.
  def self.strip(path)
    stripped = path.to_s.sub(PREFIX, "")
    stripped.start_with?("/") ? stripped : "/#{stripped}"
  end
end
