puts "Clearing old reviews..."
Review.destroy_all

puts "Creating reviews..."
Review.create!([
                 {
                   name: "Diviš poutník z Milfrau",
                   stars: 5,
                   text: "Byl jsem velmi překvapen. Ochota, vstřícnost a zboží jsme obdrželi dříve než bylo původně naplánováno.",
                   reviewed_on: Date.current - 2.months,
                   position: 1,
                   avatar: "reviews/divispoutnik.png"
                 },
                 {
                   name: "Pišta Debef",
                   stars: 5,
                   text: "Bez problémů, z katru trámy aji nejsou moc vymačkané a řez je poměrně hladký oproti jiné pile.",
                   reviewed_on: Date.current - 3.years,
                   position: 2,
                   avatar: "reviews/pistadebef.png"
                 },
                 {
                   name: "Rostislav Juracek",
                   stars: 5,
                   text: "Vstřícné a profesionální jednání",
                   reviewed_on: Date.current - 5.years,
                   position: 3,
                   avatar: "reviews/rostislavjuracek.png"
                 },
                 {
                   name: "Miroslav Hirschl",
                   stars: 5,
                   text: "Super jednání, od jiného nechci",
                   reviewed_on: Date.current - 8.years,
                   position: 4,
                   avatar: "reviews/miroslavhirschl.png"
                 }
               ])

puts "Successfully created #{Review.count} reviews!"
