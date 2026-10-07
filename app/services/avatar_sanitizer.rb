# Turns an untrusted uploaded picture into a safe profile picture.
#
# We never store or serve what the user sent. The upload is checked by its
# actual bytes (not the filename or the Content-Type the browser claims),
# limited in size and pixel count (decompression bombs), decoded by libvips
# with an allowlist of image loaders, and then re-encoded as a fresh
# 256x256 WebP. Re-encoding drops everything that isn't pixels: EXIF/GPS
# metadata, comments, trailing bytes and any payload hidden in a polyglot file.
class AvatarSanitizer
  class Invalid < StandardError
    attr_reader :reason

    def initialize(reason)
      @reason = reason
      super("Invalid avatar: #{reason}")
    end
  end

  MAX_BYTES = 2.megabytes
  MAX_PIXELS = 25_000_000 # ~5000x5000; the file itself may be tiny but decode to hundreds of MB
  SIZE = 256
  ALLOWED_TYPES = %w[image/jpeg image/png image/webp].freeze
  ALLOWED_LOADERS = %w[jpegload pngload webpload].freeze

  # Refuse libvips loaders it flags as unsafe for untrusted data (e.g. via
  # external delegates), on top of our own allowlist.
  Vips.block_untrusted(true)

  def self.call(path)
    new(path).call
  end

  def initialize(path)
    @path = path.to_s
  end

  # Returns the sanitized image as WebP bytes, or raises Invalid.
  def call
    raise Invalid, :unreadable unless File.file?(@path)
    raise Invalid, :too_large if File.size(@path) > MAX_BYTES
    raise Invalid, :unsupported_type unless ALLOWED_TYPES.include?(sniffed_type)

    check_header!
    thumbnail.webpsave_buffer(Q: 85, strip: true)
  rescue Vips::Error
    raise Invalid, :unreadable
  end

  private

  # What the bytes say the file is. Marcel reads the magic numbers only.
  def sniffed_type
    File.open(@path, "rb") { |file| Marcel::MimeType.for(file) }
  end

  # Opening is lazy: this only parses the header, it doesn't decode pixels.
  def check_header!
    image = Vips::Image.new_from_file(@path, fail_on: :error)

    raise Invalid, :unsupported_type unless ALLOWED_LOADERS.include?(image.get("vips-loader"))
    raise Invalid, :too_many_pixels if image.width * image.height > MAX_PIXELS
  end

  # Shrinks while decoding (so a huge image never lives in memory at full
  # size), crops to a centred square and applies the EXIF orientation.
  def thumbnail
    Vips::Image.thumbnail(@path, SIZE, height: SIZE, size: :both, crop: :centre, fail_on: :error)
  end
end
