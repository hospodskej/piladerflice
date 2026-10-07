require "test_helper"

class RobotsTxtTest < ActionDispatch::IntegrationTest
  PRIVATE_PATHS = %w[/admin /kosik /muj-ucet /prihlaseni /registrace /de/kosik /de/muj-ucet /de/prihlaseni /de/registrace].freeze

  test "robots.txt is served as plain text and blocks the private areas for every crawler" do
    get "/robots.txt"

    assert_response :success
    assert_equal "text/plain", response.media_type
    assert_match(/^User-agent: \*$/, response.body)
    PRIVATE_PATHS.each { |path| assert_match(/^Disallow: #{Regexp.escape(path)}$/, response.body) }
  end

  test "robots.txt points crawlers at the sitemap using the host they asked for" do
    get "/robots.txt"
    assert_match(%r{^Sitemap: http://www\.example\.com/sitemap\.xml$}, response.body)

    host! "shop.example.cz"
    get "/robots.txt"
    assert_match(%r{^Sitemap: http://shop\.example\.cz/sitemap\.xml$}, response.body)
  end

  test "robots.txt keeps the rules directly under User-agent, with no blank line splitting the group" do
    get "/robots.txt"

    assert_match(/^User-agent: \*\nDisallow: \/admin$/, response.body)
    assert_no_match(/^Disallow:.*\n\s*\n+Disallow:/, response.body)
  end

  test "robots.txt doesn't block public pages, assets or uploaded images" do
    get "/robots.txt"
    disallowed = response.body.scan(/^Disallow: (\S+)$/).flatten

    %w[/ /eshop /eshop/tramy /kontakt /sluzby /sortiment /assets/application.css /rails/active_storage/blobs/x].each do |path|
      assert disallowed.none? { |rule| path.start_with?(rule) }, "#{path} must stay crawlable"
    end
  end

  test "every blocked path still exists as a route" do
    # Routes inside the language scope are written "(/:locale)/kosik": /kosik is the Czech form, /de/kosik the German one.
    app_paths = Rails.application.routes.routes.flat_map do |route|
      spec = route.path.spec.to_s
      [spec, spec.sub("(/:locale)", ""), spec.sub("(/:locale)", "/de")]
    end

    PRIVATE_PATHS.each do |path|
      assert app_paths.any? { |route_path| route_path.start_with?(path) }, "#{path} is in robots.txt but no route starts with it"
    end
  end
end
