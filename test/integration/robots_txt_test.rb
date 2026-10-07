require "test_helper"

class RobotsTxtTest < ActionDispatch::IntegrationTest
  PRIVATE_PATHS = %w[/admin /kosik /muj-ucet /prihlaseni /registrace].freeze

  test "robots.txt is served as plain text and blocks the private areas for every crawler" do
    get "/robots.txt"

    assert_response :success
    assert_equal "text/plain", response.media_type
    assert_match(/^User-agent: \*$/, response.body)
    PRIVATE_PATHS.each { |path| assert_match(/^Disallow: #{Regexp.escape(path)}$/, response.body) }
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
    app_paths = Rails.application.routes.routes.map { |route| route.path.spec.to_s }

    PRIVATE_PATHS.each do |path|
      assert app_paths.any? { |route_path| route_path.start_with?(path) }, "#{path} is in robots.txt but no route starts with it"
    end
  end
end
