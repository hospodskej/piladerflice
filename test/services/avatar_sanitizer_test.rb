require "test_helper"

class AvatarSanitizerTest < ActiveSupport::TestCase
  def sanitize(file)
    AvatarSanitizer.call(file.path)
  end

  def assert_invalid(reason, file)
    error = assert_raises(AvatarSanitizer::Invalid) { sanitize(file) }
    assert_equal reason, error.reason
  end

  def decode(bytes)
    Vips::Image.new_from_buffer(bytes, "")
  end

  %w[jpg png webp].each do |ext|
    test "accepts a #{ext} and returns a 256x256 WebP" do
      image = decode(sanitize(image_file(ext, width: 640, height: 360)))

      assert_equal "webpload_buffer", image.get("vips-loader")
      assert_equal [256, 256], [image.width, image.height]
    end
  end

  test "a small image is scaled up to the square too" do
    image = decode(sanitize(image_file("png", width: 40, height: 40)))

    assert_equal [256, 256], [image.width, image.height]
  end

  test "keeps the EXIF metadata of the original" do
    output = sanitize(image_file("jpg", exif: "Merta-Sawmill"))

    assert_includes decode(output).get("exif-ifd0-Copyright"), "Merta-Sawmill"
  end

  test "a jpeg without EXIF still works" do
    assert_empty decode(sanitize(image_file("png"))).get_fields.grep(/exif-ifd0-Copyright/)
  end

  test "drops a payload appended to a valid image (polyglot file)" do
    payload = "<script>alert(document.cookie)</script><?php system($_GET['c']); ?>"
    upload = raw_file("jpg", File.binread(image_file("jpg").path) + payload)

    output = sanitize(upload)

    assert_not_includes output, "<script"
    assert_not_includes output, "<?php"
  end

  test "EXIF is kept, but a payload appended after it still isn't" do
    payload = "<script>alert(1)</script>"
    upload = raw_file("jpg", File.binread(image_file("jpg", exif: "keep-me").path) + payload)

    output = sanitize(upload)

    assert_includes decode(output).get("exif-ifd0-Copyright"), "keep-me"
    assert_not_includes output, "<script"
  end

  test "trusts the bytes, not the file extension" do
    png_named_exe = raw_file("exe", File.binread(image_file("png").path))

    assert_equal [256, 256], [decode(sanitize(png_named_exe)).width, decode(sanitize(png_named_exe)).height]
  end

  test "rejects a script disguised as an image" do
    assert_invalid :unsupported_type, raw_file("png", "<?php system($_GET['c']); ?>")
    assert_invalid :unsupported_type, raw_file("jpg", "<html><script>alert(1)</script></html>")
  end

  test "rejects SVG, which can carry scripts" do
    svg = %(<svg xmlns="http://www.w3.org/2000/svg" onload="alert(1)"><script>alert(1)</script></svg>)

    assert_invalid :unsupported_type, raw_file("svg", svg)
    assert_invalid :unsupported_type, raw_file("png", svg)
  end

  test "rejects formats we don't allow even though they are images" do
    assert_invalid :unsupported_type, image_file("gif")
  end

  test "the loader allowlist is a second line of defence if the content check were fooled" do
    sanitizer = AvatarSanitizer.new(image_file("gif").path)
    sanitizer.define_singleton_method(:sniffed_type) { "image/png" } # pretend the sniffing was fooled

    error = assert_raises(AvatarSanitizer::Invalid) { sanitizer.call }
    assert_equal :unsupported_type, error.reason
  end

  test "rejects an empty file" do
    assert_invalid :unsupported_type, raw_file("png", "")
  end

  test "rejects a file over the size limit" do
    noise = Vips::Image.gaussnoise(1800, 1800).cast(:uchar)
    big = raw_file("png", "")
    noise.write_to_file(big.path)
    assert_operator File.size(big.path), :>, AvatarSanitizer::MAX_BYTES

    assert_invalid :too_large, big
  end

  test "rejects a decompression bomb: tiny file, enormous pixel count" do
    bomb = image_file("png", width: 8000, height: 8000)
    assert_operator File.size(bomb.path), :<, AvatarSanitizer::MAX_BYTES

    assert_invalid :too_many_pixels, bomb
  end

  test "rejects a truncated or corrupt image" do
    valid = File.binread(image_file("png", width: 300, height: 300).path)

    assert_invalid :unreadable, raw_file("png", valid.first(valid.bytesize / 2))
    assert_invalid :unreadable, raw_file("jpg", "\xFF\xD8\xFF\xE0".b + ("garbage" * 50))
  end

  test "rejects a path that doesn't exist" do
    assert_raises(AvatarSanitizer::Invalid) { AvatarSanitizer.call("/nonexistent/file.png") }
  end
end
