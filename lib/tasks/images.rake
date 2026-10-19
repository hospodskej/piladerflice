namespace :images do
  desc "Swaps the big product and promo photos the seeds stored for the smaller copies (run once)"
  task shrink_seeded: :environment do
    # The photos the seeds attached before the site's images were shrunk, by
    # their checksum, and the smaller copy of each in app/assets/images now.
    # Matching on the exact checksum means a photo an admin uploaded is never
    # touched, even if it has the same file name.
    originals = {
      "/Q1ds5l/CAs0wVNXDVIPAA==" => "akcni-nabidka-hero.webp",
      "wfox5hpBHksBAkGRKX12BQ==" => "eshop/akat.webp",
      "ECcXojKsqo53xTiJa/0XBA==" => "eshop/bedna.webp",
      "gq6iJXwYVwRvzec099pb/Q==" => "eshop/briza.webp",
      "xDj5h9d3D52wwX/dd0HEQA==" => "eshop/buk.webp",
      "iVx++CDdZGUr0mOszC7asA==" => "eshop/container.webp",
      "S76HR5sgdQXcNZWEFbXcBg==" => "eshop/dub.webp",
      "kKrZqvL+1gnZeQKxsU8TNA==" => "eshop/fosny.webp",
      "JmNGw8e0tmKuB0lyiN8O9w==" => "eshop/habr.webp",
      "CX4qpsoOnAbN20Pg3gQ0ZQ==" => "eshop/jasan.webp",
      "xC01lXn48XZ7CCzxKC6oDw==" => "eshop/late.webp",
      "UlXDF3k59wpGL6zpGp9GXQ==" => "eshop/prkna.webp",
      "LMIGG3f2V3bzr1amCq596Q==" => "eshop/smrk.webp",
      "gKuFhXYU7NIgucIfgVYcWA==" => "eshop/tramy.webp"
    }

    swapped = saved = 0
    ActiveStorage::Blob.where(checksum: originals.keys).find_each do |old_blob|
      path = Rails.root.join("app/assets/images", originals.fetch(old_blob.checksum))
      new_blob = File.open(path, "rb") do |io|
        ActiveStorage::Blob.create_and_upload!(io: io, filename: old_blob.filename.to_s, content_type: old_blob.content_type)
      end

      old_blob.attachments.each { |attachment| attachment.update!(blob: new_blob) }
      swapped += 1
      saved += old_blob.byte_size - new_blob.byte_size
      puts "#{old_blob.filename}: #{old_blob.byte_size / 1024} KB -> #{new_blob.byte_size / 1024} KB"
      old_blob.purge
    end

    puts swapped.zero? ? "Nothing to do: no photo from the old seeds is stored." : "#{swapped} photos, #{saved / 1024} KB saved."
  end
end
