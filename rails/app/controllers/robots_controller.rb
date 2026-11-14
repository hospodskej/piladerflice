# Served by a controller rather than public/robots.txt because the Sitemap line
# must be an absolute URL, and the host isn't known until a request arrives.
class RobotsController < ApplicationController
  def show
    expires_in 1.hour, public: true
  end
end
