class CreateInquiries < ActiveRecord::Migration[8.1]
  def change
    create_table :inquiries do |t|
      t.string :first_name
      t.string :last_name
      t.string :email
      t.string :phone

      t.string :street
      t.string :house_number
      t.string :city
      t.string :zip

      # The kalkulace form's dynamic rows (variant/wood/length/amount),
      # captured as submitted rather than validated against the catalog -
      # this form intentionally lets someone ask for something outside the
      # standard product list (a custom cut, an unusual length), so unlike
      # the cart/checkout system, there's no CatalogVariant behind these
      # values to look up. JSON array of plain hashes, e.g.
      # [{"varianta"=>"Sypané", "druh"=>"Dub", "delka"=>"1 m", "mnozstvi"=>"5 PRM"}].
      t.text :items_snapshot, null: false

      t.text :notes

      # Which language the form was filled out in - the select options
      # (variant/wood/length/amount) are plain text, not translation keys,
      # so whatever language was active when submitted is what's stored;
      # this is shown alongside the raw values so whoever reads the
      # request later knows what they're looking at.
      t.string :locale, null: false, default: "cs"

      t.timestamps
    end
  end
end
