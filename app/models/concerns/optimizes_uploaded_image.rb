module OptimizesUploadedImage
  extend ActiveSupport::Concern

  OPTIMIZABLE_CONTENT_TYPES = %w[image/png image/jpeg].freeze

  OPTIMIZER = ImageOptim.new(pngout: false, advpng: false, pngcrush: false, optipng: false, svgo: false)

  def image=(attachable)
    optimize!(attachable)
    super
  end

  private

  def optimize!(attachable)
    return unless attachable.respond_to?(:tempfile) && attachable.respond_to?(:content_type)
    return unless OPTIMIZABLE_CONTENT_TYPES.include?(attachable.content_type)

    OPTIMIZER.optimize_image!(attachable.tempfile.path)
  end
end
