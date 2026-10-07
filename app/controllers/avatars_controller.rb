# The signed-in user's profile picture.
#
# Pictures are only ever shown to their owner, so instead of Active Storage's
# public (signed-URL) blob routes we serve them from here, behind the login.
# What is stored is always the WebP produced by AvatarSanitizer, never the
# original upload.
class AvatarsController < ApplicationController
  before_action :require_login

  # Uploading runs libvips, so keep it cheap to hammer.
  UPLOAD_LIMIT_STORE = ActiveSupport::Cache::MemoryStore.new
  rate_limit to: 10, within: 10.minutes, only: :update, store: UPLOAD_LIMIT_STORE,
             with: -> { redirect_to edit_account_path, alert: t("auth.avatar_errors.rate_limited") }

  def show
    return head :not_found unless current_user.avatar.attached?

    blob = current_user.avatar.blob
    # The URL carries ?v=<checksum>, so a changed picture is a new URL.
    expires_in 1.year, public: false
    response.headers["X-Content-Type-Options"] = "nosniff"
    response.headers["Content-Security-Policy"] = "default-src 'none'; sandbox"
    send_data blob.download, type: "image/webp", disposition: "inline", filename: "avatar.webp"
  end

  def update
    upload = params[:avatar]
    return redirect_to edit_account_path, alert: t("auth.avatar_errors.none_selected") unless upload.respond_to?(:tempfile)

    webp = AvatarSanitizer.call(upload.tempfile.path)
    current_user.avatar.attach(io: StringIO.new(webp), filename: "avatar.webp", content_type: "image/webp", identify: false)
    redirect_to edit_account_path, notice: t("auth.avatar_updated")
  rescue AvatarSanitizer::Invalid => error
    redirect_to edit_account_path, alert: t("auth.avatar_errors.#{error.reason}")
  end

  def destroy
    current_user.avatar.purge
    redirect_to edit_account_path, notice: t("auth.avatar_removed")
  end
end
