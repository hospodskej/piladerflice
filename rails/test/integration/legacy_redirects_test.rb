require "test_helper"

# The old WordPress site's addresses keep working after the switch.
class LegacyRedirectsTest < ActionDispatch::IntegrationTest
  {
    "/obchod/" => "/eshop",
    "/obchod/stavebni-rezivo/" => "/eshop?category=rezivo",
    "/produkt/palivove-drevo-tvrde-akat/" => "/eshop/akat",
    "/produkt/hranoly/" => "/eshop/tramy",
    "/produkt/okrasne-kamenivo/" => "/eshop/okrasne_kamenivo",
    "/zakazkova-vyroba/" => "/kontakt#kalkulace",
    "/zasady-cookies-eu/" => "/zasady-cookies",
    "/de/" => "/at",
    "/de/sortiment/brennholz/" => "/at/sortiment/palivove-drevo",
    "/de/eshop/ziersteine-eshop/" => "/at/eshop?category=kamenivo",
    "/de/zur-kasse/" => "/at/kosik",
    "/produkt/neco-co-uz-neni/" => "/eshop",
    "/de/irgendwas/" => "/at"
  }.each do |old_path, new_path|
    test "#{old_path} redirects permanently to #{new_path}" do
      get old_path
      assert_response :moved_permanently
      assert_equal "http://www.example.com#{new_path}", response.location
    end
  end

  test "every product the old site linked to is in the catalogue seeds" do
    seeds = Rails.root.join("db/seeds_catalog.rb").read
    targets = Rails.root.join("config/routes/legacy.rb").read.scan(%r{=> "/eshop/([a-z_]+)"}).flatten
    assert_operator targets.size, :>=, 15
    targets.each do |key|
      assert_includes seeds, %(key: "#{key}"), "/eshop/#{key} has no product in the seeds"
    end
  end

  test "www redirects to the bare domain with the same path" do
    with_app_host("piladerflice.cz") do
      host! "www.piladerflice.cz"
      get "/sortiment/?x=1"
      assert_response :moved_permanently
      assert_equal "https://piladerflice.cz/sortiment?x=1", response.location
    end
  end

  test "addresses the new site kept still answer directly" do
    %w[/kontakt/ /sluzby/ /sortiment/palivove-drevo/ /kosik/].each do |path|
      get path
      assert_response :success, path
    end
  end

  private

  # The www redirect is only drawn when APP_HOST is set, as in production.
  def with_app_host(host)
    original = ENV["APP_HOST"]
    ENV["APP_HOST"] = host
    Rails.application.reload_routes!
    yield
  ensure
    ENV["APP_HOST"] = original
    Rails.application.reload_routes!
  end
end
