module Admin
  class BaseController < ApplicationController
    include Authentication

    layout "admin"

    before_action :require_admin
    before_action :no_store
    before_action { response.headers["X-Robots-Tag"] = "noindex, nofollow" }

    private

    def require_admin
      return if current_user&.admin?

      redirect_to root_path, alert: "Nemáte oprávnění k přístupu do administrace."
    end
  end
end
