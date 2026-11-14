# Content-Security-Policy: the browser runs only our own scripts (and Google
# Analytics), so injected HTML can't run a script or send data elsewhere.
# See https://guides.rubyonrails.org/security.html#content-security-policy-header

Rails.application.configure do
  # Where Google Analytics sends its measurements (www.google.com/g/collect
  # with consent mode).
  google_analytics = %w[https://*.google-analytics.com https://*.analytics.google.com https://*.googletagmanager.com https://www.google.com]

  config.content_security_policy do |policy|
    policy.default_src :self
    policy.script_src  :self, "https://www.googletagmanager.com"
    policy.connect_src :self, *google_analytics
    policy.img_src     :self, :data, *google_analytics
    # style="" attributes are used throughout; styles can't run code.
    policy.style_src   :self, :unsafe_inline
    policy.font_src    :self, :data
    policy.frame_src   "https://www.google.com" # the map in the footer
    policy.object_src  :none
    policy.base_uri    :self
    policy.form_action :self
    policy.frame_ancestors :self
  end

  # Our few inline scripts (theme, Google Analytics, the purchase event) carry
  # this nonce. On pages it is random and stays the same for the whole visit,
  # because Turbo keeps the first page's policy while it swaps pages. Other
  # responses (sitemap, robots.txt, images) get a one-off one, so they don't
  # start a session.
  config.content_security_policy_nonce_generator = lambda do |request|
    if request.format.html?
      request.session[:csp_nonce] ||= SecureRandom.base64(18)
    else
      SecureRandom.base64(18)
    end
  end
  config.content_security_policy_nonce_directives = %w[script-src]
end
