module Admin
  class PricelistItemsController < BaseController
    before_action :set_pricelist_item, only: [:update, :destroy, :move_up, :move_down]

    def index
      @items = PricelistItem.ordered.group_by(&:category)
    end

    def create
      @pricelist_item = PricelistItem.new(pricelist_item_params)
      @pricelist_item.position = PricelistItem.where(category: @pricelist_item.category).maximum(:position).to_i + 1

      if @pricelist_item.save
        redirect_to admin_pricelist_items_path(anchor: @pricelist_item.category), notice: "Položka byla přidána."
      else
        redirect_to admin_pricelist_items_path(anchor: @pricelist_item.category), alert: @pricelist_item.errors.full_messages.join(", ")
      end
    end

    def update
      if @pricelist_item.update(pricelist_item_params.except(:category))
        redirect_to admin_pricelist_items_path(anchor: @pricelist_item.category), notice: "Položka byla uložena."
      else
        redirect_to admin_pricelist_items_path(anchor: @pricelist_item.category), alert: @pricelist_item.errors.full_messages.join(", ")
      end
    end

    def destroy
      @pricelist_item.destroy
      redirect_to admin_pricelist_items_path(anchor: @pricelist_item.category), notice: "Položka byla smazána."
    end

    def move_up
      move(-1)
    end

    def move_down
      move(1)
    end

    private

    def set_pricelist_item
      @pricelist_item = PricelistItem.find(params[:id])
    end

    def pricelist_item_params
      params.require(:pricelist_item).permit(:category, :item_name, :item_name_de, :details, :details_de,
                                             :subcategory, :subcategory_de, :price, :price_de)
    end

    # Renumbers the whole category while swapping, so items that share a
    # position (e.g. created by the seed with the default 0) still move.
    def move(direction)
      siblings = PricelistItem.where(category: @pricelist_item.category).ordered.to_a
      index = siblings.index(@pricelist_item)
      target = index + direction

      if target.between?(0, siblings.size - 1)
        siblings[index], siblings[target] = siblings[target], siblings[index]
        PricelistItem.transaction do
          siblings.each_with_index { |item, position| item.update_column(:position, position) }
        end
      end

      redirect_to admin_pricelist_items_path(anchor: @pricelist_item.category)
    end
  end
end
