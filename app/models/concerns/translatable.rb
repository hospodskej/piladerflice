module Translatable
  extend ActiveSupport::Concern

  class_methods do
    # Declares that +attrs+ have a matching "<attr>_de" column, and defines
    # a "<attr>_i18n" reader on each record that returns the German value
    # when browsing in German, falling back to the Czech column if no
    # translation has been filled in yet (so nothing renders blank while
    # content is still being translated).
    def translates(*attrs)
      attrs.each do |attr|
        define_method("#{attr}_i18n") do
          if I18n.locale == :de
            de_value = read_attribute("#{attr}_de")
            de_value.presence || read_attribute(attr)
          else
            read_attribute(attr)
          end
        end
      end
    end
  end
end
