require "test_helper"

class AvatarsControllerTest < ActionDispatch::IntegrationTest
  setup do
    AvatarsController::UPLOAD_LIMIT_STORE.clear
    @user = User.create!(email_address: "zakaznik@example.com", password: "supersecret", role: "customer")
    @other = User.create!(email_address: "jiny@example.com", password: "supersecret", role: "customer")
  end

  def sign_in(email = "zakaznik@example.com")
    post session_path, params: { email_address: email, password: "supersecret" }
  end

  def upload(file, type: "image/png")
    patch avatar_path, params: { avatar: Rack::Test::UploadedFile.new(file.path, type) }
  end

  test "everything requires a login" do
    get avatar_path
    assert_redirected_to login_path

    patch avatar_path
    assert_redirected_to login_path

    delete avatar_path
    assert_redirected_to login_path
  end

  test "uploading stores only a re-encoded 256x256 WebP under a fixed name, keeping its EXIF" do
    sign_in
    upload(image_file("jpg", width: 900, height: 500, exif: "Merta-Sawmill"), type: "image/jpeg")

    assert_redirected_to edit_account_path
    assert_equal I18n.t("auth.avatar_updated"), flash[:notice]

    avatar = @user.reload.avatar
    assert avatar.attached?
    assert_equal "image/webp", avatar.content_type
    assert_equal "avatar.webp", avatar.filename.to_s

    image = Vips::Image.new_from_buffer(avatar.download, "")
    assert_equal [256, 256], [image.width, image.height]
    assert_includes image.get("exif-ifd0-Copyright"), "Merta-Sawmill"
  end

  test "ignores the filename and content type the browser claims" do
    sign_in
    upload(raw_file("php", File.binread(image_file("png").path)), type: "application/x-php")

    assert @user.reload.avatar.attached?
    assert_equal "avatar.webp", @user.avatar.filename.to_s
  end

  test "rejects something that isn't an image and keeps the old picture" do
    sign_in
    upload(image_file("png"))
    old_checksum = @user.reload.avatar.blob.checksum

    upload(raw_file("png", "<?php system($_GET['c']); ?>"))

    assert_redirected_to edit_account_path
    assert_equal I18n.t("auth.avatar_errors.unsupported_type"), flash[:alert]
    assert_equal old_checksum, @user.reload.avatar.blob.checksum
  end

  test "rejects SVG, oversized files and decompression bombs with a clear message" do
    sign_in

    upload(raw_file("svg", %(<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>)), type: "image/svg+xml")
    assert_equal I18n.t("auth.avatar_errors.unsupported_type"), flash[:alert]

    upload(image_file("png", width: 8000, height: 8000))
    assert_equal I18n.t("auth.avatar_errors.too_many_pixels"), flash[:alert]

    noise = raw_file("png", "")
    Vips::Image.gaussnoise(1800, 1800).cast(:uchar).write_to_file(noise.path)
    upload(noise)
    assert_equal I18n.t("auth.avatar_errors.too_large"), flash[:alert]

    assert_not @user.reload.avatar.attached?
  end

  test "submitting without a file shows a message instead of an error" do
    sign_in
    patch avatar_path

    assert_redirected_to edit_account_path
    assert_equal I18n.t("auth.avatar_errors.none_selected"), flash[:alert]
  end

  test "a text field can't stand in for the file" do
    sign_in
    patch avatar_path, params: { avatar: "not-a-file" }

    assert_equal I18n.t("auth.avatar_errors.none_selected"), flash[:alert]
    assert_not @user.reload.avatar.attached?
  end

  test "replacing the picture swaps the stored blob" do
    sign_in
    upload(image_file("png", width: 300, height: 300))
    first = @user.reload.avatar.blob

    upload(image_file("jpg", width: 500, height: 500), type: "image/jpeg")

    assert_not_equal first.id, @user.reload.avatar.blob.id
    assert_equal 1, ActiveStorage::Attachment.where(record: @user, name: "avatar").count
  end

  test "show serves the owner's picture with locked-down headers" do
    sign_in
    upload(image_file("png"))
    get avatar_path

    assert_response :success
    assert_equal "image/webp", response.media_type
    assert_equal @user.reload.avatar.download, response.body
    assert_equal "nosniff", response.headers["X-Content-Type-Options"]
    assert_match(/default-src 'none'/, response.headers["Content-Security-Policy"])
    assert_match(/\Ainline/, response.headers["Content-Disposition"])
    assert_match(/private/, response.headers["Cache-Control"])
  end

  test "show is a 404 when there is no picture" do
    sign_in
    get avatar_path

    assert_response :not_found
  end

  test "you only ever get your own picture" do
    sign_in
    upload(image_file("png"))
    delete logout_path

    sign_in("jiny@example.com")
    get avatar_path

    assert_response :not_found
  end

  test "an id in the request can't be used to fetch someone else's picture" do
    sign_in
    upload(image_file("png"))
    delete logout_path
    sign_in("jiny@example.com")

    get avatar_path(id: @user.id, user_id: @user.id)

    assert_response :not_found
  end

  test "removing the picture deletes it" do
    sign_in
    upload(image_file("png"))
    assert @user.reload.avatar.attached?

    delete avatar_path

    assert_redirected_to edit_account_path
    assert_not @user.reload.avatar.attached?
    assert_equal 0, ActiveStorage::Attachment.where(record: @user, name: "avatar").count
  end

  test "uploading is rate limited" do
    AvatarsController::UPLOAD_LIMIT_STORE.clear
    sign_in
    10.times { upload(image_file("png", width: 50, height: 50)) }

    upload(image_file("png", width: 50, height: 50))

    assert_equal I18n.t("auth.avatar_errors.rate_limited"), flash[:alert]
  ensure
    AvatarsController::UPLOAD_LIMIT_STORE.clear
  end

  test "the account pages show the picture, and the header uses it" do
    sign_in
    upload(image_file("png"))

    get account_path
    assert_select ".account-avatar img" do |images|
      assert images.first["src"].start_with?("/muj-ucet/profilovy-obrazek?v=")
    end
    assert_select "a.account-btn img.account-btn-avatar"

    get edit_account_path
    assert_select ".avatar-box input[type=file][accept=?]", "image/jpeg,image/png,image/webp"
    assert_select ".avatar-box form[enctype=?]", "multipart/form-data"
  end

  test "without a picture the pages fall back to the initial and the icon" do
    sign_in

    get account_path
    assert_select ".account-avatar img", false
    assert_select ".account-avatar", /Z/
    assert_select "a.account-btn img", false
  end

  test "the German edit page is translated" do
    sign_in
    get edit_account_path, params: { locale: "de" }

    assert_select ".avatar-box .checkout-box-title", "Profilbild"
  end
end
