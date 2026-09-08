module Admin
  class BaseController < ApplicationController
    include Authentication

    layout "admin"
  end
end
