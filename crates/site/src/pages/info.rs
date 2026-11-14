//! The services page and the legal pages.

use super::home::Service;
#[cfg(feature = "ssr")]
use super::home::load_services;
use super::{PageMeta, SafeHtml, loaded};
use crate::i18n::t;
use crate::islands::SlideShow;
use crate::layout::Layout;
use crate::visitor::visitor;
use leptos::prelude::*;

/// Rails' `simple_format`: blank lines start a new paragraph, a single line
/// break becomes <br>.
pub fn simple_format(html: &SafeHtml) -> String {
    let text = html.as_str().replace("\r\n", "\n");
    text.split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| format!("<p>{}</p>", p.replace('\n', "\n<br />")))
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[server(endpoint = "page/sluzby")]
pub async fn services_page_data() -> Result<Vec<Service>, ServerFnError> {
    let v = visitor();
    let db = &expect_context::<crate::state::AppState>().db;
    load_services(db, &v, Some("/sluzby"))
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// The anchor of a service section: what its button points to after "#"
/// (#doprava), or its title made into an address.
fn section_id(service: &Service) -> String {
    service
        .button_path
        .as_deref()
        .and_then(|p| p.rsplit('#').next())
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| parameterize(&service.title))
}

/// "Pořez / Prodej kulatiny" → "porez-prodej-kulatiny".
pub fn parameterize(text: &str) -> String {
    let ascii: String = text
        .chars()
        .map(|c| match c {
            'á' | 'Á' => 'a',
            'č' | 'Č' => 'c',
            'ď' | 'Ď' => 'd',
            'é' | 'ě' | 'É' | 'Ě' => 'e',
            'í' | 'Í' => 'i',
            'ň' | 'Ň' => 'n',
            'ó' | 'Ó' | 'ö' | 'Ö' => 'o',
            'ř' | 'Ř' => 'r',
            'š' | 'Š' => 's',
            'ť' | 'Ť' => 't',
            'ú' | 'ů' | 'Ú' | 'Ů' | 'ü' | 'Ü' => 'u',
            'ý' | 'Ý' => 'y',
            'ž' | 'Ž' => 'z',
            'ä' | 'Ä' => 'a',
            'ß' => 's',
            c => c.to_ascii_lowercase(),
        })
        .collect();
    ascii
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[component]
pub fn SluzbyPage() -> impl IntoView {
    let l = visitor().locale;
    view! {
        <PageMeta title=t::meta::sluzby::title(l).to_owned() description=t::meta::sluzby::description(l)/>
        <Layout>
            <main class="main">
                <section class="sluzby-page-section">
                    {loaded(services_page_data(), move |services: Vec<Service>| {
                        let v = visitor();
                        services.into_iter().enumerate().map(|(index, s)| {
                            let id = section_id(&s);
                            let title = s.title.clone();
                            view! {
                                <div class=if index % 2 == 1 { "sluzba-row reverse-layout" } else { "sluzba-row" } id=id>
                                    <div class="sluzba-content">
                                        <h2>{s.title.clone()}</h2>
                                        <div class="sluzba-text" inner_html=simple_format(&s.content)></div>
                                        {s.button.map(|(text, _)| view! {
                                            <a href=v.link("/kontakt#cenik-sluzeb") class="btn-primary">{text}</a>
                                        })}
                                    </div>
                                    {(!s.images.is_empty()).then(|| view! {
                                        <SlideShow>
                                            <div class="sluzba-carousel-wrapper">
                                                <button class="carousel-btn prev-btn" aria-label=t::common::previous(l)>
                                                    <svg viewBox="0 0 24 24" fill="white"><path d="M15.41 16.59L10.83 12l4.58-4.59L14 6l-6 6 6 6 1.41-1.41z"/></svg>
                                                </button>
                                                <div class="carousel-track-container">
                                                    <ul class="carousel-track">
                                                        {s.images.into_iter().enumerate().map(|(i, src)| {
                                                            let first = index == 0 && i == 0;
                                                            view! {
                                                                <li class=if i == 0 { "carousel-slide active" } else { "carousel-slide" }>
                                                                    <img src=src alt=title.clone() loading=if first { "eager" } else { "lazy" } fetchpriority=first.then_some("high")/>
                                                                </li>
                                                            }
                                                        }).collect_view()}
                                                    </ul>
                                                </div>
                                                <button class="carousel-btn next-btn" aria-label=t::common::next(l)>
                                                    <svg viewBox="0 0 24 24" fill="white"><path d="M8.59 16.59L13.17 12 8.59 7.41 10 6l6 6-6 6-1.41-1.41z"/></svg>
                                                </button>
                                            </div>
                                        </SlideShow>
                                    })}
                                </div>
                            }
                        }).collect_view()
                    })}
                </section>
            </main>
        </Layout>
    }
}

fn legal_page(
    title: &'static str,
    meta_title: &'static str,
    description: &'static str,
    content: codon::Html,
) -> impl IntoView {
    view! {
        <PageMeta title=meta_title.to_owned() description=description/>
        <Layout>
            <main class="main">
                <section class="legal-page">
                    <h1>{title}</h1>
                    <div class="legal-content" inner_html=content.into_string()></div>
                </section>
            </main>
        </Layout>
    }
}

#[component]
pub fn TermsPage() -> impl IntoView {
    let v = visitor();
    let l = v.locale;
    legal_page(
        t::legal::terms_title(l),
        t::meta::terms::title(l),
        t::meta::terms::description(l),
        t::legal::terms_content(l, v.shop.czk_per_km),
    )
}

#[component]
pub fn CookiesPage() -> impl IntoView {
    let l = visitor().locale;
    legal_page(
        t::legal::cookies_title(l),
        t::meta::cookies::title(l),
        t::meta::cookies::description(l),
        t::legal::cookies_content(l),
    )
}

#[component]
pub fn PrivacyPage() -> impl IntoView {
    let l = visitor().locale;
    legal_page(
        t::legal::privacy_title(l),
        t::meta::privacy::title(l),
        t::meta::privacy::description(l),
        t::legal::privacy_content(l),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameterizes_czech() {
        assert_eq!(
            parameterize("Pořez / Prodej kulatiny"),
            "porez-prodej-kulatiny"
        );
    }
}
