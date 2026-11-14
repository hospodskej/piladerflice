module Admin
  class InquiriesController < BaseController
    def index
      @inquiries = Inquiry.order(created_at: :desc)
    end

    def show
      @inquiry = Inquiry.find(params[:id])
    end
  end
end
