class AvatarSanitizer
  # What the three image formats share: reading numbers out of the file,
  # where anything missing or out of place means a broken file, and the
  # pixel limit.
  class Format
    def initialize(bytes)
      @bytes = bytes.b
    end

    # The cleaned file's bytes; raises Invalid.
    def clean
      raise NotImplementedError
    end

    private

    def unreadable!
      raise Invalid, :unreadable
    end

    def bytes_at(offset, length)
      unreadable! if offset.negative? || offset + length > @bytes.bytesize
      @bytes.byteslice(offset, length)
    end

    def byte_at(offset) = bytes_at(offset, 1).getbyte(0)
    def u16be(offset) = bytes_at(offset, 2).unpack1("n")
    def u32be(offset) = bytes_at(offset, 4).unpack1("N")
    def u16le(offset) = bytes_at(offset, 2).unpack1("v")
    def u32le(offset) = bytes_at(offset, 4).unpack1("V")

    # Refuses an empty image or one too big to decode safely.
    def check_dimensions!(width, height)
      unreadable! if width.zero? || height.zero?
      raise Invalid, :too_many_pixels if width * height > MAX_PIXELS
    end
  end
end
