require "base64"
require "zlib"

# Builds image files for the tests in plain Ruby, so they need no image
# library and the repo no binary fixtures. PNGs are drawn for real; JPEG and
# WebP start from a tiny real 16x16 picture whose header is then given the
# size a test asks for (the app reads sizes from headers, it never decodes).
module ImageFixtures
  TINY_JPEG = Base64.decode64(
    "/9j/2wBDAA0JCgsKCA0LCgsODg0PEyAVExISEyccHhcgLikxMC4pLSwzOko+MzZGNywtQFdBRkxOUlNSMj5aYVpQYEpRUk//2wBDAQ4ODhMR" \
    "EyYVFSZPNS01T09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT0//wAARCAAQABADASIAAhEBAxEB/8QA" \
    "FQABAQAAAAAAAAAAAAAAAAAAAAP/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/8QAFAEBAAAAAAAAAAAAAAAAAAAABf/EABQRAQAAAAAAAAAAAAAA" \
    "AAAAAAD/2gAMAwEAAhEDEQA/AKAATb//2Q=="
  ).freeze
  TINY_WEBP = Base64.decode64("UklGRjYAAABXRUJQVlA4ICoAAACwAQCdASoQABAAAsBMJaACdLoABdQAAP3HH/hWuYfmHtb/3BmfV1/gAAA=").freeze
  TINY_WEBP_LOSSLESS = Base64.decode64("UklGRh4AAABXRUJQVlA4TBEAAAAvD8ADAAdQvCIXpf+BiOh/AAA=").freeze
  TINY_GIF = Base64.decode64("R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7").freeze

  # A picture file; `exif` (JPEG only) adds an EXIF block with this
  # copyright, a GPS position and `orientation`.
  def image_file(ext, width: 400, height: 300, exif: nil, orientation: nil)
    bytes = case ext
    when "png" then png_bytes(width:, height:)
    when "jpg" then jpeg_bytes(width:, height:, exif:, orientation:)
    when "webp" then webp_bytes(width:, height:)
    when "gif" then TINY_GIF
    end
    raw_file(ext, bytes)
  end

  def raw_file(ext, bytes)
    file = Tempfile.new(["upload", ".#{ext}"])
    file.binmode
    file.write(bytes)
    file.close
    (@temp_image_files ||= []) << file
    file
  end

  # A real orange PNG; very large sizes get only a header and a token of
  # data (enough for the size checks, which never decode).
  def png_bytes(width:, height:, extra_chunks: [], idat: nil)
    idat ||= if width * height <= 4_000_000
      row = "\0".b + ([200, 120, 40].pack("C3") * width)
      Zlib::Deflate.deflate(row * height)
    else
      Zlib::Deflate.deflate("\0" * 64)
    end
    ihdr = [width, height, 8, 2, 0, 0, 0].pack("N N C C C C C")
    "\x89PNG\r\n\x1A\n".b + png_chunk("IHDR", ihdr) + extra_chunks.sum("".b) { |type, data| png_chunk(type, data) } +
      png_chunk("IDAT", idat) + png_chunk("IEND", "")
  end

  def png_chunk(type, data)
    [data.bytesize].pack("N") + type.b + data.b + [Zlib.crc32(type + data)].pack("N")
  end

  def jpeg_bytes(width:, height:, exif: nil, orientation: nil)
    bytes = TINY_JPEG.dup
    frame = bytes.index("\xFF\xC0".b)
    bytes[frame + 5, 4] = [height, width].pack("n n")
    bytes.insert(2, exif_segment(exif, orientation)) if exif || orientation
    bytes
  end

  # An APP1 EXIF block (big-endian TIFF): orientation, copyright and a GPS
  # position in IFD0.
  def exif_segment(copyright, orientation)
    copyright = "#{copyright}\0"
    entries = []
    entries << [0x0112, 3, 1, [orientation, 0].pack("n n")] if orientation
    copyright_at = 8 + 2 + (3 * 12) + 4
    entries << [0x8298, 2, copyright.bytesize, [copyright_at].pack("N")]
    gps_at = copyright_at + copyright.bytesize
    entries << [0x8825, 4, 1, [gps_at].pack("N")]
    ifd = [entries.size].pack("n") + entries.sum("".b) { |tag, type, count, value| [tag, type, count].pack("n n N") + value }
    ifd = ifd.ljust(2 + (3 * 12), "\0") + [0].pack("N")
    gps = [1].pack("n") + [1, 2, 2].pack("n n N") + "N\0\0\0".b + [0].pack("N")
    tiff = "MM\0*".b + [8].pack("N") + ifd + copyright.b + gps
    data = "Exif\0\0".b + tiff
    "\xFF\xE1".b + [data.bytesize + 2].pack("n") + data
  end

  # A lossy WebP; `lossless` uses VP8L. `extra_chunks` ([type, data]) go
  # in an extended (VP8X) file before the picture.
  def webp_bytes(width:, height:, lossless: false, extra_chunks: [])
    source = lossless ? TINY_WEBP_LOSSLESS : TINY_WEBP
    type = source.byteslice(12, 4)
    picture = source.byteslice(20, source.byteslice(16, 4).unpack1("V")).dup
    if lossless
      bits = picture.byteslice(1, 4).unpack1("V") & ~((1 << 28) - 1)
      picture[1, 4] = [bits | (width - 1) | ((height - 1) << 14)].pack("V")
    else
      picture[6, 4] = [width, height].pack("v v")
    end
    chunks = [[type, picture]]
    unless extra_chunks.empty?
      header = [0x2C, 0, 0, 0].pack("C4") + [width - 1].pack("V").byteslice(0, 3) + [height - 1].pack("V").byteslice(0, 3)
      chunks = [["VP8X", header], *extra_chunks, *chunks]
    end
    body = chunks.sum("".b) { |chunk_type, data| chunk_type.b + [data.bytesize].pack("V") + data + ("\0" * (data.bytesize & 1)) }
    "RIFF".b + [body.bytesize + 4].pack("V") + "WEBP".b + body
  end
end
