require "test_helper"

class InquiriesControllerTest < ActionDispatch::IntegrationTest
  def valid_params
    {
      jmeno: "Jana", prijmeni: "Nováková", email: "jana@example.com", telefon: "123456789",
      ulice: "Hlavní", cislo_popisne: "12", mesto: "Brno", psc: "60200", poznamky: "Prosím zavolat předem",
      kalkulace_category: "palivove",
      varianta: [ "Sypané" ], druh: [ "Dub" ], delka: [ "1 m" ], mnozstvi: [ "5 PRM" ]
    }
  end

  test "create saves an inquiry and emails a notification" do
    assert_emails 1 do
      assert_difference -> { Inquiry.count }, 1 do
        post inquiries_path, params: valid_params
      end
    end

    assert_redirected_to kontakt_path(anchor: "kalkulace")

    inquiry = Inquiry.last
    assert_equal "Jana", inquiry.first_name
    assert_equal "Brno", inquiry.city
    assert_equal "palivove", inquiry.category
    assert_equal [ { "varianta" => "Sypané", "druh" => "Dub", "delka" => "1 m", "mnozstvi" => "5 PRM" } ], inquiry.items
  end

  test "create saves a stavebni inquiry with its own field set" do
    params = valid_params.merge(
      kalkulace_category: "stavebni",
      polozka: [ "Trám" ], delka: [ "6 m" ], vyska: [ "33 cm" ], sirka: [ "33 cm" ], mnozstvi: [ "25" ]
    )

    post inquiries_path, params: params

    inquiry = Inquiry.last
    assert_equal "stavebni", inquiry.category
    assert_equal [ { "polozka" => "Trám", "delka" => "6 m", "vyska" => "33 cm", "sirka" => "33 cm", "mnozstvi" => "25" } ], inquiry.items
  end

  test "create accepts a freely typed custom value in place of a dropdown selection" do
    params = valid_params.merge(varianta: [ "Moje vlastní varianta" ])
    post inquiries_path, params: params

    assert_equal "Moje vlastní varianta", Inquiry.last.items.first["varianta"]
  end

  test "create falls back to the palivove category for an unknown or missing kalkulace_category" do
    post inquiries_path, params: valid_params.except(:kalkulace_category)
    assert_equal "palivove", Inquiry.last.category

    post inquiries_path, params: valid_params.merge(kalkulace_category: "bogus")
    assert_equal "palivove", Inquiry.last.category
  end

  test "create redirects with an error when required fields are missing" do
    assert_no_difference -> { Inquiry.count } do
      post inquiries_path, params: valid_params.except(:telefon)
    end

    assert_redirected_to kontakt_path(anchor: "kalkulace")
    assert_equal I18n.t("kontakt.inquiry_error"), flash[:alert]
  end
end
