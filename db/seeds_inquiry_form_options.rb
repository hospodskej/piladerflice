puts "Clearing old inquiry form options..."
InquiryFormOption.destroy_all

puts "Creating inquiry form options (palivove, stavebni)..."

palivove = {
  "varianta" => [
    { value: "Skládané", value_de: "Gestapelt" },
    { value: "Sypané", value_de: "Lose" }
  ],
  "druh" => [
    { value: "Akát", value_de: "Akazie" },
    { value: "Dub", value_de: "Eiche" },
    { value: "Smrk", value_de: "Fichte" },
    { value: "Buk", value_de: "Buche" },
    { value: "Jasan", value_de: "Esche" },
    { value: "Habr", value_de: "Hainbuche" },
    { value: "Bříza", value_de: "Birke" }
  ],
  "delka" => [
    { value: "33 cm", value_de: "33 cm" },
    { value: "50 cm", value_de: "50 cm" },
    { value: "1 m", value_de: "1 m" }
  ],
  "mnozstvi" => [
    { value: "5 PRM", value_de: "5 Rm" },
    { value: "10 PRM", value_de: "10 Rm" },
    { value: "15 PRM", value_de: "15 Rm" }
  ]
}

stavebni = {
  "polozka" => [
    { value: "Trám", value_de: "Balken" },
    { value: "Prkno", value_de: "Brett" },
    { value: "Fošna", value_de: "Bohle" },
    { value: "Latě", value_de: "Latte" }
  ],
  "delka" => [
    { value: "6 m", value_de: "6 m" },
    { value: "3 m", value_de: "3 m" },
    { value: "4 m", value_de: "4 m" },
    { value: "5 m", value_de: "5 m" }
  ],
  "vyska" => [
    { value: "33 cm", value_de: "33 cm" },
    { value: "50 cm", value_de: "50 cm" },
    { value: "100 cm", value_de: "100 cm" }
  ],
  "sirka" => [
    { value: "33 cm", value_de: "33 cm" },
    { value: "50 cm", value_de: "50 cm" },
    { value: "100 cm", value_de: "100 cm" }
  ]
}

{ "palivove" => palivove, "stavebni" => stavebni }.each do |category, fields|
  fields.each do |field, options|
    options.each_with_index do |option, index|
      InquiryFormOption.create!(
        category: category, field: field, position: index,
        value: option[:value], value_de: option[:value_de]
      )
    end
  end
end

puts "Inquiry form options seeded: #{InquiryFormOption.count}."
