//! The price list tables (Ceník), from the rows the admin edits.

use leptos::prelude::*;

use crate::ui::t;

/// A row; its texts are HTML (the admin may write m³, line breaks...).
pub struct PriceRow {
    pub name: String,
    pub details: String,
    pub price: String,
    /// "---" means "no price", shown without the "/ m³" suffix.
    pub has_price: bool,
}

/// Firewood rows of one kind of wood.
pub struct PriceGroup {
    /// HTML.
    pub name: String,
    pub items: Vec<PriceRow>,
}

pub struct PriceList {
    pub stavebni: Vec<PriceRow>,
    pub palivove_volne: Vec<PriceGroup>,
    pub palivove_skladane: Vec<PriceGroup>,
    pub zbytky: Vec<PriceRow>,
    /// One price, HTML; None shows the default text.
    pub kamenivo: Option<String>,
    pub sluzby: Vec<PriceRow>,
}

#[component]
pub fn PriceListView(prices: PriceList) -> impl IntoView {
    let PriceList {
        stavebni,
        palivove_volne,
        palivove_skladane,
        zbytky,
        kamenivo,
        sluzby,
    } = prices;
    view! {
        <section class="cenik-section" id="cenik">
            <h2 class="cenik-main-title">{t("kontakt.cenik_main_title")}</h2>
            <Group title_key="kontakt.cenik_stavebni_title">
                <table class="cenik-table">
                    <tbody>
                        {stavebni
                            .into_iter()
                            .map(|row| {
                                view! {
                                    <tr>
                                        <td inner_html=row.name></td>
                                        <td inner_html=row.details></td>
                                        <td inner_html=row.price></td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </Group>
            <Firewood title_key="kontakt.cenik_palivove_volne_title" groups=palivove_volne/>
            <Firewood title_key="kontakt.cenik_palivove_skladane_title" groups=palivove_skladane/>
            <Group title_key="kontakt.cenik_zbytky_title">
                <table class="cenik-table two-cols">
                    <tbody>
                        {zbytky
                            .into_iter()
                            .map(|row| {
                                view! {
                                    <tr>
                                        <td style="text-align: left" inner_html=row.name></td>
                                        <td inner_html=row.price></td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </Group>
            <Group title_key="kontakt.cenik_kamenivo_title">
                <table class="cenik-table">
                    <tbody>
                        <tr>
                            <td
                                style="text-align: right; padding-right: 30px"
                                inner_html=kamenivo.unwrap_or_else(|| t("kontakt.cenik_kamenivo_default"))
                            ></td>
                        </tr>
                    </tbody>
                </table>
            </Group>
            <h2 class="cenik-main-title" id="cenik-sluzeb" style="margin-top: 80px; margin-bottom: 20px">
                {t("kontakt.cenik_sluzeb_title")}
            </h2>
            <div class="cenik-sluzby-container">
                {sluzby
                    .into_iter()
                    .map(|service| {
                        view! {
                            <div class="cenik-sluzba-item">
                                <h4 class="cenik-sluzba-label" inner_html=service.name></h4>
                                <div class="cenik-sluzba-box" inner_html=service.price></div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

/// A table with its heading.
#[component]
fn Group(title_key: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="cenik-group">
            <div class="cenik-header">
                <h3>{t(title_key)}</h3>
                <span class="cenik-line"></span>
            </div>
            {children()}
        </div>
    }
}

/// Firewood: a small table per kind of wood, prices per m³.
#[component]
fn Firewood(title_key: &'static str, groups: Vec<PriceGroup>) -> impl IntoView {
    let per_m3 = t("common.per_m3");
    view! {
        <Group title_key=title_key>
            {groups
                .into_iter()
                .map(|group| {
                    let per_m3 = per_m3.clone();
                    view! {
                        <div class="cenik-subgroup">
                            <h4 class="cenik-subtitle" inner_html=group.name></h4>
                            <table class="cenik-table two-cols">
                                <tbody>
                                    {group
                                        .items
                                        .into_iter()
                                        .map(|item| {
                                            let price = if item.has_price {
                                                format!("{} / {}", item.price, sapwood_i18n::escape_html(&per_m3))
                                            } else {
                                                item.price
                                            };
                                            view! {
                                                <tr>
                                                    <td inner_html=item.name></td>
                                                    <td inner_html=price></td>
                                                </tr>
                                            }
                                        })
                                        .collect_view()}
                                </tbody>
                            </table>
                        </div>
                    }
                })
                .collect_view()}
        </Group>
    }
}
