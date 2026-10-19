ENV["RAILS_ENV"] ||= "test"
require_relative "../config/environment"
require "rails/test_help"
require_relative "support/fake_route_service"
require_relative "support/image_fixtures"

module ActiveSupport
  class TestCase
    include ImageFixtures
    include FakeRouteService

    teardown { restore_route_service }

    # Run tests in parallel with specified workers
    parallelize(workers: 1)

    # Setup all fixtures in test/fixtures/*.yml for all tests in alphabetical order.
    fixtures :all

    # Add more helper methods to be used by all tests here...
  end
end
