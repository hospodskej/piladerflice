require "test_helper"

class SecurityTest < ActionDispatch::IntegrationTest
  def login(email: "admin@piladerflice.cz", password: "correct-horse-battery", ip: "203.0.113.5")
    post session_path, params: { email_address: email, password: password }, env: { "REMOTE_ADDR" => ip }
  end

  # --- Repeated attempts ---------------------------------------------------

  test "password guessing from one address is stopped after 10 tries in 3 minutes" do
    10.times { login(password: "wrong") }
    assert_response :unprocessable_entity

    login # even the right password
    assert_redirected_to root_path
    assert_equal "Příliš mnoho pokusů za krátkou dobu. Zkuste to prosím za pár minut znovu.", flash[:alert]
    assert_nil cookies[:session_id].presence

    travel 4.minutes do
      login
      assert_redirected_to account_path
    end
  end

  test "guessing one account's password from many addresses is stopped too" do
    20.times { |i| login(password: "wrong", ip: "198.51.100.#{i}") }
    login(ip: "198.51.100.99")
    assert_equal I18n.t("security.too_many_attempts", locale: :cs), flash[:alert]
  end

  test "registering, guessing discount codes, inquiries, delivery lookups and orders are limited" do
    6.times { |i| post registrations_path, params: { user: { email_address: "spam#{i}@example.com", password: "supersecret", password_confirmation: "supersecret" } } }
    assert_equal 5, User.where("email_address LIKE 'spam%'").count

    post cart_items_path, params: { catalog_variant_id: catalog_variants(:tramy_variant).id }
    11.times { |i| post cart_discount_path, params: { code: "GUESS#{i}" } }
    assert_equal I18n.t("security.too_many_attempts", locale: :cs), flash[:alert]

    inquiry = { jmeno: "A", prijmeni: "B", email: "a@b.cz", telefon: "1", items: { palivove: { druh: ["Dub"] } } }
    assert_difference -> { Inquiry.count }, 5 do
      6.times { post inquiries_path, params: inquiry }
    end

    fake_route
    21.times { |i| post delivery_estimate_path, params: { query: "Obec #{i}", country: "cz" } }
    assert_equal 20, @fake_route_urls.count { |url| url.include?("/geocode/") }
  end

  # --- Authentication ------------------------------------------------------

  test "signing in gives a new session ID but keeps the cart" do
    post cart_items_path, params: { catalog_variant_id: catalog_variants(:tramy_variant).id }
    before = session.id.to_s

    login
    assert_redirected_to account_path
    assert_not_equal before, session.id.to_s
    assert_equal 1, session[:cart].size
  end

  test "signing out ends the login and empties the browser session" do
    login
    post cart_items_path, params: { catalog_variant_id: catalog_variants(:tramy_variant).id }

    delete logout_path
    assert_nil session[:cart]
    get account_path
    assert_redirected_to login_path
  end

  test "an admin login expires after 30 days, a customer's doesn't" do
    login
    get admin_root_path
    assert_response :success

    travel 31.days do
      get admin_root_path
      assert_redirected_to login_path
    end

    User.create!(email_address: "zakaznik@example.com", password: "supersecret", role: "customer")
    login(email: "zakaznik@example.com", password: "supersecret", ip: "203.0.113.9")
    travel 400.days do
      get account_path
      assert_response :success
    end
  end

  test "after signing in, only a path on this site is followed" do
    ["//evil.example", "/\\evil.example", "https://evil.example", "/\tevil", "javascript:alert(1)"].each do |target|
      reset!
      get login_path, params: { return_to: target }
      login
      assert_redirected_to account_path, "followed #{target.inspect}"
    end

    reset!
    get login_path, params: { return_to: "/kosik" }
    login
    assert_redirected_to "/kosik"
  end

  # --- Misconfiguration ----------------------------------------------------

  test "pages send a Content-Security-Policy and every inline script carries its nonce" do
    get root_path
    policy = response.headers["Content-Security-Policy"]
    nonce = policy[/'nonce-([^']+)'/, 1]

    assert_includes policy, "default-src 'self'"
    assert_includes policy, "object-src 'none'"
    assert_includes policy, "frame-ancestors 'self'"
    assert nonce.present?
    scripts = css_select("script:not([src]):not([type='application/ld+json']):not([type='importmap']), script[src*='googletagmanager']")
    assert scripts.any?
    scripts.each { |script| assert_equal nonce, script["nonce"] }

    # The same nonce for the whole visit (Turbo keeps the first page's policy).
    get eshop_path
    assert_equal nonce, response.headers["Content-Security-Policy"][/'nonce-([^']+)'/, 1]
  end

  test "browser features the site doesn't use are switched off" do
    get root_path
    assert_includes response.headers["Permissions-Policy"], "camera=()"
    assert_includes response.headers["Permissions-Policy"], "geolocation=()"
  end

  test "pages with personal data aren't cached, and the admin isn't indexed" do
    login
    get admin_root_path
    assert_equal "no-store", response.headers["Cache-Control"]
    assert_equal "noindex, nofollow", response.headers["X-Robots-Tag"]

    get account_path
    assert_equal "no-store", response.headers["Cache-Control"]
  end

  test "anyone storing a file through Active Storage's direct uploads is refused" do
    assert_no_difference -> { ActiveStorage::Blob.count } do
      post "/rails/active_storage/direct_uploads",
           params: { blob: { filename: "x.html", byte_size: 4, checksum: "ICy5YqxZB1uWSwcVLSNLcA==", content_type: "text/html" } },
           as: :json
    end
    assert_response :not_found

    put "/rails/active_storage/disk/anything"
    assert_response :not_found
  end

  # --- Injection -----------------------------------------------------------

  test "texts from the database can format but never run a script" do
    services(:one).update!(content: "Doprava<br>po celé Moravě<script>alert(1)</script><img src=x onerror=alert(2)>")

    get root_path
    assert_includes response.body, "Doprava<br>po celé Moravě"
    assert_not_includes response.body, "<script>alert(1)"
    assert_not_includes response.body, "onerror"
  end

  test "search-like input can't change a database query" do
    get eshop_path, params: { category: "palivove' OR '1'='1", wood: "x'; DROP TABLE users; --", sort: "') OR 1=1 --" }
    assert_response :success
    assert User.exists?

    post cart_discount_path, params: { code: "' OR 1=1 --" }
    assert_nil session[:discount_code]
  end
end
