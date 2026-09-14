require "test_helper"

class InquiriesControllerTest < ActionDispatch::IntegrationTest
  def valid_params
    {
      jmeno: "Jana", prijmeni: "Nováková", email: "jana@example.com", telefon: "123456789",
      ulice: "Hlavní", cislo_popisne: "12", mesto: "Brno", psc: "60200", poznamky: "Prosím zavolat předem",
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
    assert_equal [ { "varianta" => "Sypané", "druh" => "Dub", "delka" => "1 m", "mnozstvi" => "5 PRM" } ], inquiry.items
  end

  test "create redirects with an error when required fields are missing" do
    assert_no_difference -> { Inquiry.count } do
      post inquiries_path, params: valid_params.except(:telefon)
    end

    assert_redirected_to kontakt_path(anchor: "kalkulace")
    assert_equal I18n.t("kontakt.inquiry_error"), flash[:alert]
  end
end
