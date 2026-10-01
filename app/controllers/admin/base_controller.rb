module Admin
  class BaseController < ApplicationController
    include Authentication

    layout "admin"

    before_action :require_admin

    private

    def require_admin
      return if current_user&.admin?

      redirect_to root_path, alert: "Nemáte oprávnění k přístupu do administrace."
    end
  end
end
