class CheckoutState
  SESSION_KEY = :checkout

  STRING_ATTRIBUTES = %w[
    shipping_method payment_method country
    first_name last_name email phone
    billing_street billing_city billing_zip billing_country
    company_name company_ico company_dic
    delivery_street delivery_city delivery_zip delivery_country
  ].freeze

  BOOLEAN_ATTRIBUTES = %w[
    newsletter_opt_in company_purchase delivery_address_different
  ].freeze

  ATTRIBUTES = (STRING_ATTRIBUTES + BOOLEAN_ATTRIBUTES).freeze

  def initialize(session)
    @session = session
    @session[SESSION_KEY] ||= {}
  end

  def [](key)
    @session[SESSION_KEY][key.to_s]
  end

  def update(attrs)
    attrs.to_h.each do |key, value|
      key = key.to_s
      next unless ATTRIBUTES.include?(key)

      @session[SESSION_KEY][key] = BOOLEAN_ATTRIBUTES.include?(key) ? boolean(value) : value
    end
  end

  def newsletter_opt_in?
    boolean(self["newsletter_opt_in"])
  end

  def company_purchase?
    boolean(self["company_purchase"])
  end

  def delivery_address_different?
    boolean(self["delivery_address_different"])
  end

  def shipping_selected?
    self["shipping_method"].present? && self["payment_method"].present?
  end

  def details_complete?
    %w[first_name last_name email phone billing_street billing_city billing_zip billing_country].all? do |key|
      self[key].present?
    end
  end

  def clear
    @session[SESSION_KEY] = {}
  end

  private

  def boolean(value)
    ActiveModel::Type::Boolean.new.cast(value)
  end
end
