require "test_helper"

class AvatarSanitizerTest < ActiveSupport::TestCase
  def sanitize(file)
    AvatarSanitizer.call(file.path)
  end

  def assert_invalid(reason, file)
    error = assert_raises(AvatarSanitizer::Invalid) { sanitize(file) }
    assert_equal reason, error.reason
  end

  { "jpg" => "image/jpeg", "png" => "image/png", "webp" => "image/webp" }.each do |ext, type|
    test "accepts a #{ext} and keeps its type" do
      result = sanitize(image_file(ext, width: 640, height: 360))

      assert_equal type, result.content_type
      assert_equal ext, result.extension
      assert_equal type, Marcel::MimeType.for(StringIO.new(result.bytes))
    end
  end

  test "a clean picture comes back unchanged" do
    png = image_file("png", width: 40, height: 40)
    assert_equal File.binread(png.path), sanitize(png).bytes

    webp = image_file("webp", width: 40, height: 40)
    assert_equal File.binread(webp.path), sanitize(webp).bytes
  end

  test "a JPEG loses its EXIF (camera, GPS) but keeps the orientation" do
    output = sanitize(image_file("jpg", exif: "Merta-Sawmill", orientation: 6)).bytes

    assert_not_includes output, "Merta-Sawmill"
    assert_equal 6, AvatarSanitizer::Exif.orientation(output.byteslice(output.index("Exif"), 100))
    assert_equal 1, output.scan("Exif".b).size
  end

  test "a JPEG standing upright needs no EXIF at all" do
    output = sanitize(image_file("jpg", exif: "Merta-Sawmill")).bytes

    assert_not_includes output, "Exif"
    assert_equal File.binread(image_file("jpg").path), output
  end

  test "a JPEG's comments are dropped" do
    jpeg = jpeg_bytes(width: 16, height: 16)
    comment = "\xFF\xFE".b + [2 + 13].pack("n") + "secret-note!!"
    output = sanitize(raw_file("jpg", jpeg.dup.insert(2, comment))).bytes

    assert_not_includes output, "secret-note"
  end

  test "a PNG loses its text and EXIF chunks, keeps its colour profile" do
    png = png_bytes(width: 20, height: 20, extra_chunks: [
      ["tEXt", "Author\0Merta"], ["eXIf", "MM\0*secret-gps"], ["gAMA", [45_455].pack("N")]
    ])
    output = sanitize(raw_file("png", png)).bytes

    assert_not_includes output, "Merta"
    assert_not_includes output, "secret-gps"
    assert_includes output, "gAMA"
  end

  test "a WebP loses its EXIF and XMP, keeps its colour profile" do
    webp = webp_bytes(width: 20, height: 20, extra_chunks: [
      ["ICCP", "profile-data"], ["EXIF", "MM\0*secret-gps"], ["XMP ", "<x:xmpmeta>Merta</x:xmpmeta>"]
    ])
    output = sanitize(raw_file("webp", webp)).bytes

    assert_not_includes output, "secret-gps"
    assert_not_includes output, "xmpmeta"
    assert_includes output, "profile-data"
    assert_equal 0x20, output.byteslice(20, 1).unpack1("C"), "the header says: a colour profile, nothing else"
  end

  test "lossless WebPs work too" do
    webp = raw_file("webp", webp_bytes(width: 30, height: 20, lossless: true))

    assert_equal File.binread(webp.path), sanitize(webp).bytes
  end

  %w[jpg png webp].each do |ext|
    test "drops a payload appended to a #{ext} (polyglot file)" do
      payload = "<script>alert(document.cookie)</script><?php system($_GET['c']); ?>"
      upload = raw_file(ext, File.binread(image_file(ext).path) + payload)

      output = sanitize(upload).bytes

      assert_not_includes output, "<script"
      assert_not_includes output, "<?php"
    end
  end

  test "trusts the bytes, not the file extension" do
    png_named_exe = raw_file("exe", File.binread(image_file("png").path))

    assert_equal "image/png", sanitize(png_named_exe).content_type
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

  test "rejects animated WebPs" do
    webp = webp_bytes(width: 20, height: 20, extra_chunks: [["ANIM", "\0" * 6]])

    assert_invalid :unsupported_type, raw_file("webp", webp)
  end

  test "a file pretending to be another format doesn't get through" do
    sanitizer = AvatarSanitizer.new(image_file("gif").path)
    sanitizer.define_singleton_method(:sniffed_type) { "image/png" } # pretend the sniffing was fooled

    error = assert_raises(AvatarSanitizer::Invalid) { sanitizer.call }
    assert_equal :unreadable, error.reason
  end

  test "rejects an empty file" do
    assert_invalid :unsupported_type, raw_file("png", "")
  end

  test "rejects a file over the size limit" do
    big = raw_file("png", png_bytes(width: 2000, height: 2000, idat: Random.bytes(AvatarSanitizer::MAX_BYTES)))

    assert_invalid :too_large, big
  end

  %w[jpg png webp].each do |ext|
    test "rejects a #{ext} decompression bomb: tiny file, enormous pixel count" do
      bomb = image_file(ext, width: 8000, height: 8000)
      assert_operator File.size(bomb.path), :<, AvatarSanitizer::MAX_BYTES

      assert_invalid :too_many_pixels, bomb
    end
  end

  test "rejects a truncated or corrupt image" do
    %w[jpg png webp].each do |ext|
      valid = File.binread(image_file(ext, width: 300, height: 300).path)
      assert_invalid :unreadable, raw_file(ext, valid.byteslice(0, valid.bytesize / 2))
    end
    assert_invalid :unreadable, raw_file("jpg", "\xFF\xD8\xFF\xE0".b + ("garbage" * 50))
  end

  test "rejects a PNG whose checksums don't match" do
    png = png_bytes(width: 20, height: 20)
    png[-20] = (png.getbyte(-20) ^ 0xFF).chr # a byte of the picture data

    assert_invalid :unreadable, raw_file("png", png)
  end

  test "rejects a path that doesn't exist" do
    assert_raises(AvatarSanitizer::Invalid) { AvatarSanitizer.call("/nonexistent/file.png") }
  end
end
