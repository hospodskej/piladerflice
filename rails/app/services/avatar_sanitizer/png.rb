require "zlib"

class AvatarSanitizer
  # Rebuilds a PNG from its chunks. Kept: those that draw the picture (IHDR,
  # PLTE, IDAT, IEND) and those that change how it looks (transparency,
  # gamma, colour space and profile). Left out: text and EXIF chunks, the
  # time stamp, animation (the first frame stays) and anything unknown or
  # after the end.
  class Png < Format
    SIGNATURE = "\x89PNG\r\n\x1A\n".b
    KEEP = %w[IHDR PLTE IDAT IEND tRNS gAMA cHRM sRGB iCCP sBIT].freeze

    def clean
      unreadable! unless @bytes.start_with?(SIGNATURE)

      kept = SIGNATURE.dup
      position = SIGNATURE.bytesize
      types = []
      until types.last == "IEND"
        length = u32be(position)
        chunk = bytes_at(position, length + 12)
        type = chunk.byteslice(4, 4)
        unreadable! unless Zlib.crc32(chunk.byteslice(4, length + 4)) == chunk.byteslice(-4, 4).unpack1("N")

        if types.empty?
          unreadable! unless type == "IHDR" && length == 13
          check_dimensions!(*chunk.byteslice(8, 8).unpack("N N"))
        end
        kept << chunk if KEEP.include?(type)
        types << type
        position += chunk.bytesize
      end
      unreadable! unless types.include?("IDAT")

      kept
    end
  end
end
