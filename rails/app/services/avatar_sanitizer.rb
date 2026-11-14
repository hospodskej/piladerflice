# Turns an untrusted uploaded picture into a safe profile picture, in plain
# Ruby (no image library to install).
#
# We never store what the user sent. The upload is identified by its actual
# bytes (not the filename or the Content-Type the browser claims) and must be
# a JPEG, PNG or WebP of at most 2 MB and 25 megapixels. The pixel count is
# read from the file's header, so a small file that would decode to gigabytes
# (a decompression bomb) is refused without being decoded.
#
# The file is then rebuilt from only the parts a browser needs to show it:
# metadata (EXIF with the camera and the GPS position, XMP, IPTC, comments,
# text chunks) and anything after the end of the image (e.g. a script hidden
# in a polyglot file) are left out. A JPEG keeps its EXIF orientation alone,
# so phone photos still stand the right way up.
#
# The pixels themselves are never decoded or re-encoded, so the picture keeps
# its size and the pages scale it down. It is only ever served to its owner,
# as an image, with nosniff and a sandboxing CSP (AvatarsController).
class AvatarSanitizer
  class Invalid < StandardError
    attr_reader :reason

    def initialize(reason)
      @reason = reason
      super("Invalid avatar: #{reason}")
    end
  end

  # The cleaned picture: its bytes, type and file extension.
  Result = Data.define(:bytes, :content_type, :extension)

  MAX_BYTES = 2.megabytes
  MAX_PIXELS = 25_000_000 # ~5000x5000; the file itself may be tiny but decode to hundreds of MB

  FORMATS = {
    "image/jpeg" => [Jpeg, "jpg"],
    "image/png" => [Png, "png"],
    "image/webp" => [Webp, "webp"]
  }.freeze

  def self.call(path)
    new(path).call
  end

  def initialize(path)
    @path = path.to_s
  end

  # Returns a Result, or raises Invalid.
  def call
    raise Invalid, :unreadable unless File.file?(@path)
    raise Invalid, :too_large if File.size(@path) > MAX_BYTES

    type = sniffed_type
    format, extension = FORMATS[type]
    raise Invalid, :unsupported_type unless format

    Result.new(bytes: format.new(File.binread(@path)).clean, content_type: type, extension: extension)
  end

  private

  # What the bytes say the file is. Marcel reads the magic numbers only.
  def sniffed_type
    File.open(@path, "rb") { |file| Marcel::MimeType.for(file) }
  end
end
