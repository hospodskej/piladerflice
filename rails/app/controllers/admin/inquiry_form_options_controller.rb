module Admin
  class InquiryFormOptionsController < BaseController
    before_action :set_inquiry_form_option, only: [:update, :destroy, :move_up, :move_down]

    def index
      @options = InquiryFormOption.ordered.group_by(&:category).transform_values { |options| options.group_by(&:field) }
    end

    def create
      @inquiry_form_option = InquiryFormOption.new(inquiry_form_option_params)
      @inquiry_form_option.position = next_position(@inquiry_form_option.category, @inquiry_form_option.field)

      if @inquiry_form_option.save
        redirect_to admin_inquiry_form_options_path(anchor: anchor_for(@inquiry_form_option)), notice: "Možnost byla přidána."
      else
        redirect_to admin_inquiry_form_options_path, alert: @inquiry_form_option.errors.full_messages.to_sentence
      end
    end

    def update
      if @inquiry_form_option.update(inquiry_form_option_params.except(:category, :field))
        redirect_to admin_inquiry_form_options_path(anchor: anchor_for(@inquiry_form_option)), notice: "Možnost byla uložena."
      else
        redirect_to admin_inquiry_form_options_path(anchor: anchor_for(@inquiry_form_option)), alert: @inquiry_form_option.errors.full_messages.to_sentence
      end
    end

    def destroy
      @inquiry_form_option.destroy
      redirect_to admin_inquiry_form_options_path, notice: "Možnost byla smazána."
    end

    def move_up
      swap_with_neighbor(-1)
      redirect_to admin_inquiry_form_options_path(anchor: anchor_for(@inquiry_form_option))
    end

    def move_down
      swap_with_neighbor(1)
      redirect_to admin_inquiry_form_options_path(anchor: anchor_for(@inquiry_form_option))
    end

    private

    def set_inquiry_form_option
      @inquiry_form_option = InquiryFormOption.find(params[:id])
    end

    def inquiry_form_option_params
      params.require(:inquiry_form_option).permit(:category, :field, :value, :value_de)
    end

    def next_position(category, field)
      InquiryFormOption.where(category: category, field: field).maximum(:position).to_i + 1
    end

    def anchor_for(option)
      "#{option.category}-#{option.field}"
    end

    def swap_with_neighbor(direction)
      siblings = InquiryFormOption.for_field(@inquiry_form_option.category, @inquiry_form_option.field).to_a
      index = siblings.index(@inquiry_form_option)
      neighbor = siblings[index + direction] if index && index + direction >= 0
      return unless neighbor

      original_position = @inquiry_form_option.position

      InquiryFormOption.transaction do
        @inquiry_form_option.update!(position: neighbor.position)
        neighbor.update!(position: original_position)
      end
    end
  end
end
