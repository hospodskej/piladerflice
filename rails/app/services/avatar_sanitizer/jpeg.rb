class AvatarSanitizer
  # Rebuilds a JPEG from its segments. Kept: the tables and frame that make
  # up the picture, and APP0 (JFIF), APP2 (ICC colour profile) and APP14
  # (Adobe colour transform), which change how it looks. Left out: APP1
  # (EXIF, XMP), APP13 (IPTC), comments and every other application segment,
  # and anything after the end-of-image marker. The EXIF orientation comes
  # back as a minimal EXIF block of its own.
  class Jpeg < Format
    SOI = "\xFF\xD8".b
    EOI = "\xFF\xD9".b
    SOS = 0xDA
    APP1 = 0xE1
    # Quantization and Huffman tables, arithmetic coding conditioning, restart interval.
    TABLES = [0xDB, 0xC4, 0xCC, 0xDD].freeze
    COLOUR = [0xE0, 0xE2, 0xEE].freeze
    # Start of frame, in all its variants (C4, C8 and CC are other markers).
    FRAMES = ((0xC0..0xCF).to_a - [0xC4, 0xC8, 0xCC]).freeze
    RESTARTS = (0xD0..0xD7)

    def clean
      unreadable! unless @bytes.start_with?(SOI)

      kept = +"".b
      orientation = nil
      frame = scanned = false
      position = 2
      loop do
        marker, position = next_marker(position)
        break if marker == 0xD9
        unreadable! if RESTARTS.cover?(marker) || marker == 0x01 || marker == 0xD8

        length = u16be(position)
        unreadable! if length < 2
        segment = bytes_at(position - 2, length + 2)
        data = segment.byteslice(4..)
        position += length

        if FRAMES.include?(marker)
          unreadable! if data.bytesize < 5
          check_dimensions!(*data.unpack("x n n"))
          frame = true
          kept << segment
        elsif marker == SOS
          unreadable! unless frame
          scan_end = end_of_scan(position)
          kept << segment << @bytes.byteslice(position...scan_end)
          position = scan_end
          scanned = true
        elsif marker == APP1
          orientation ||= Exif.orientation(data)
        elsif TABLES.include?(marker) || COLOUR.include?(marker)
          kept << segment
        end
      end
      unreadable! unless scanned

      SOI + Exif.orientation_segment(orientation) + kept + EOI
    end

    private

    # The marker at `position` (after any fill bytes) and where its segment
    # starts.
    def next_marker(position)
      unreadable! unless byte_at(position) == 0xFF
      position += 1 while byte_at(position) == 0xFF
      [byte_at(position), position + 1]
    end

    # Where the compressed picture data after a start-of-scan ends: at the
    # first marker that isn't a stuffed 0xFF byte or a restart marker.
    def end_of_scan(position)
      loop do
        found = @bytes.index("\xFF".b, position) or unreadable!
        following = byte_at(found + 1)
        return found unless following.zero? || following == 0xFF || RESTARTS.cover?(following)

        position = found + (following == 0xFF ? 1 : 2)
      end
    end
  end
end
