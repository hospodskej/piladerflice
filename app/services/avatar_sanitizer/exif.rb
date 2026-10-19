class AvatarSanitizer
  # The one piece of EXIF a profile picture keeps: its orientation (how a
  # phone photo has to be turned), read from an APP1 segment and written as
  # a minimal EXIF block of its own.
  module Exif
    HEADER = "Exif\0\0".b
    ORIENTATION_TAG = 0x0112
    SHORT = 3

    module_function

    # 2..8 from an APP1 segment's data; nil when it has none (or it is 1,
    # "as stored"), or the block isn't readable.
    def orientation(data)
      return unless data.start_with?(HEADER)

      tiff = data.byteslice(HEADER.bytesize..)
      short, long = case tiff.byteslice(0, 2)
      when "II" then %w[v V]
      when "MM" then %w[n N]
      else return
      end
      ifd = tiff.byteslice(4, 4)&.unpack1(long) or return
      count = tiff.byteslice(ifd, 2)&.unpack1(short) or return
      count.times do |index|
        entry = tiff.byteslice(ifd + 2 + (index * 12), 12)
        return if entry.nil? || entry.bytesize < 12

        tag, type = entry.unpack("#{short}#{short}")
        next unless tag == ORIENTATION_TAG && type == SHORT

        value = entry.byteslice(8, 2).unpack1(short)
        return (2..8).cover?(value) ? value : nil
      end
      nil
    end

    # An APP1 segment holding only the orientation; empty without one.
    def orientation_segment(orientation)
      return "".b unless orientation

      tiff = "MM\0*".b + [8, 1, ORIENTATION_TAG, SHORT, 1, orientation, 0, 0].pack("N n n n N n n N")
      data = HEADER + tiff
      "\xFF\xE1".b + [data.bytesize + 2].pack("n") + data
    end
  end
end
