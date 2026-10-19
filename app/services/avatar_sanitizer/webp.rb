class AvatarSanitizer
  # Rebuilds a WebP from its chunks. Kept: the picture (VP8 lossy or VP8L
  # lossless), its alpha channel and its colour profile. Left out: EXIF and
  # XMP, anything unknown, and anything after the RIFF container. Animated
  # WebPs are refused.
  class Webp < Format
    PICTURES = ["VP8 ", "VP8L"].freeze
    ALPHA_FLAG = 0x10
    ICC_FLAG = 0x20

    def clean
      unreadable! unless bytes_at(0, 4) == "RIFF" && bytes_at(8, 4) == "WEBP"

      chunks = read_chunks(8 + u32le(4))
      picture_type, picture = chunks.find { |type, _| PICTURES.include?(type) } || unreadable!
      width, height = dimensions(picture_type, picture)
      check_dimensions!(width, height)
      if (canvas = chunks.assoc("VP8X"))
        check_dimensions!(*canvas_size(canvas.last))
      end

      icc = chunks.assoc("ICCP")
      alpha = chunks.assoc("ALPH")
      body = if icc || alpha
        flags = (icc ? ICC_FLAG : 0) | (alpha ? ALPHA_FLAG : 0)
        header = [flags, 0, 0, 0].pack("C4") + le24(width - 1) + le24(height - 1)
        [["VP8X", header], icc, alpha, [picture_type, picture]].compact.sum("".b) { |type, data| chunk(type, data) }
      else
        chunk(picture_type, picture)
      end
      "RIFF".b + [body.bytesize + 4].pack("V") + "WEBP".b + body
    end

    private

    # [type, data] of every chunk inside the RIFF container (which ends at `last`).
    def read_chunks(last)
      unreadable! if last > @bytes.bytesize
      chunks = []
      position = 12
      while position < last
        type = bytes_at(position, 4)
        length = u32le(position + 4)
        unreadable! if position + 8 + length > last
        raise Invalid, :unsupported_type if %w[ANIM ANMF].include?(type)

        chunks << [type, bytes_at(position + 8, length)]
        position += 8 + length + (length & 1)
      end
      chunks
    end

    def dimensions(type, data)
      if type == "VP8L"
        unreadable! unless data.getbyte(0) == 0x2F && data.bytesize >= 5
        bits = data.byteslice(1, 4).unpack1("V")
        [(bits & 0x3FFF) + 1, ((bits >> 14) & 0x3FFF) + 1]
      else
        unreadable! unless data.byteslice(3, 3) == "\x9D\x01\x2A".b && data.bytesize >= 10
        data.byteslice(6, 4).unpack("v v").map { |size| size & 0x3FFF }
      end
    end

    def canvas_size(header)
      unreadable! unless header.bytesize == 10
      [header.byteslice(4, 3), header.byteslice(7, 3)].map { |size| (size + "\0").unpack1("V") + 1 }
    end

    def le24(number) = [number].pack("V").byteslice(0, 3)

    def chunk(type, data)
      type.b + [data.bytesize].pack("V") + data + (data.bytesize.odd? ? "\0".b : "".b)
    end
  end
end
