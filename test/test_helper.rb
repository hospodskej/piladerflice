ENV["RAILS_ENV"] ||= "test"
require_relative "../config/environment"
require "rails/test_help"

# Builds real image files on the fly so the repo needs no binary fixtures.
module ImageFixtures
  def image_file(ext, width: 400, height: 300, exif: nil)
    image = (Vips::Image.black(width, height, bands: 3) + [200, 120, 40]).cast(:uchar).copy
    image.set_type(GObject::GSTR_TYPE, "exif-ifd0-Copyright", exif) if exif
    write_temp(ext) { |path| image.write_to_file(path) }
  end

  def raw_file(ext, bytes)
    write_temp(ext) { |path| File.binwrite(path, bytes) }
  end

  private

  def write_temp(ext)
    file = Tempfile.new(["upload", ".#{ext}"])
    file.close
    yield file.path
    (@temp_image_files ||= []) << file
    file
  end
end

module ActiveSupport
  class TestCase
    include ImageFixtures

    # Run tests in parallel with specified workers
    parallelize(workers: 1)

    # Setup all fixtures in test/fixtures/*.yml for all tests in alphabetical order.
    fixtures :all

    # Add more helper methods to be used by all tests here...
  end
end
