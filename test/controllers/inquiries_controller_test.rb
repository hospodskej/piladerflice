require "test_helper"

class InquiriesControllerTest < ActionDispatch::IntegrationTest
  def valid_params
    {
      jmeno: "Jana", prijmeni: "Nováková", email: "jana@example.com", telefon: "123456789",
      ulice: "Hlavní", cislo_popisne: "12", mesto: "Brno", psc: "60200", poznamky: "Prosím zavolat předem",
      items: { palivove: { varianta: [ "Sypané" ], druh: [ "Dub" ], delka: [ "1 m" ], mnozstvi: [ "5 PRM" ] } }
    }
  end

  def stavebni_items
    { polozka: [ "Trám" ], delka: [ "6 m" ], vyska: [ "33 cm" ], sirka: [ "33 cm" ], mnozstvi: [ "25" ] }
  end

  test "create saves an inquiry and emails a notification" do
    assert_emails 1 do
      assert_difference -> { Inquiry.count }, 1 do
        post inquiries_path, params: valid_params
      end
    end

    assert_redirected_to kontakt_path(anchor: "kalkulace")
    assert_equal "Děkujeme, poptávku jsme přijali. Ozveme se vám co nejdříve.", flash[:notice]

    inquiry = Inquiry.last
    assert_equal "Jana", inquiry.first_name
    assert_equal "Brno", inquiry.city
    assert_equal({ "palivove" => [ { "varianta" => "Sypané", "druh" => "Dub", "delka" => "1 m", "mnozstvi" => "5 PRM" } ] }, inquiry.items_by_category)
  end

  test "create saves a stavebni inquiry with its own field set" do
    post inquiries_path, params: valid_params.merge(items: { stavebni: stavebni_items })

    inquiry = Inquiry.last
    assert_equal %w[stavebni], inquiry.categories
    assert_equal [ { "polozka" => "Trám", "delka" => "6 m", "vyska" => "33 cm", "sirka" => "33 cm", "mnozstvi" => "25" } ],
                 inquiry.items_by_category["stavebni"]
  end

  test "one inquiry can ask for firewood and lumber together" do
    params = valid_params.deep_merge(items: { stavebni: stavebni_items })

    assert_emails 1 do
      post inquiries_path, params: params
    end

    inquiry = Inquiry.last
    assert_equal %w[palivove stavebni], inquiry.categories
    assert_equal "Palivové dřevo + Stavební řezivo", inquiry.category_labels
    assert_equal "Dub", inquiry.items_by_category["palivove"].first["druh"]
    assert_equal "Trám", inquiry.items_by_category["stavebni"].first["polozka"]

    mail = ActionMailer::Base.deliveries.last.text_part.body.to_s
    assert_includes mail, "POŽADOVANÉ POLOŽKY – PALIVOVÉ DŘEVO"
    assert_includes mail, "POŽADOVANÉ POLOŽKY – STAVEBNÍ ŘEZIVO"
    assert_includes mail, "Trám | 6 m | 33 cm | 33 cm | 25"
  end

  test "create accepts a freely typed custom value in place of a dropdown selection" do
    params = valid_params.deep_merge(items: { palivove: { varianta: [ "Moje vlastní varianta" ] } })
    post inquiries_path, params: params

    assert_equal "Moje vlastní varianta", Inquiry.last.items_by_category["palivove"].first["varianta"]
  end

  test "an inquiry with no items, or only unknown categories, is refused" do
    assert_no_difference -> { Inquiry.count } do
      post inquiries_path, params: valid_params.except(:items)
      post inquiries_path, params: valid_params.merge(items: { bogus: { druh: [ "Dub" ] } })
    end
    assert_redirected_to kontakt_path(anchor: "kalkulace")
  end

  test "create redirects with an error when required fields are missing" do
    assert_no_difference -> { Inquiry.count } do
      post inquiries_path, params: valid_params.except(:telefon)
    end

    assert_redirected_to kontakt_path(anchor: "kalkulace")
    assert_equal "Poptávku se nepodařilo odeslat. Vyplňte prosím jméno, příjmení, e-mail a telefon.", flash[:alert]
  end

  test "the form lets the customer switch both categories on" do
    get kontakt_path

    assert_select ".toggle-btn[data-target=palivove][aria-pressed=true]"
    assert_select ".toggle-btn[data-target=stavebni][aria-pressed=false]"
    assert_select ".kalkulace-toggles-hint", text: "Můžete vybrat jedno i obojí."
    assert_select "fieldset[data-category=palivove] select[name='items[palivove][druh][]']"
    assert_select "fieldset[data-category=stavebni][disabled] input[name='items[stavebni][mnozstvi][]']"
    assert_select "input[name=kalkulace_category]", count: 0
  end
end
