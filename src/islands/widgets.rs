//! Islands used across the pages: the accordion, the carousels, the
//! Google reviews and the headline whose lines always fit.

use std::time::Duration;

use leptos::html;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// One item of an accordion (services, frequent questions): the title
/// opens and closes it; `children` is the body. `faq`: the questions'
/// look (a plus sign instead of the arrow).
#[island]
pub fn AccordionItem(title: String, faq: bool, children: Children) -> impl IntoView {
    let open = RwSignal::new(false);
    let icon = if faq {
        view! { <span class="faq-icon"></span> }.into_any()
    } else {
        view! {
            <span class="accordion-icon">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="6 9 12 15 18 9"/>
                </svg>
            </span>
        }
        .into_any()
    };
    view! {
        <div class=if faq { "accordion-item faq-item" } else { "accordion-item" } class:active=open>
            <button class="accordion-header" on:click=move |_| open.update(|open| *open = !*open)>
                <span class="accordion-title">{title}</span>
                " "
                {icon}
            </button>
            <div class="accordion-content-wrapper">
                <div class="accordion-content">
                    <div class="accordion-body">{children()}</div>
                </div>
            </div>
        </div>
    }
}

/// How long a slide stays before the next one comes.
const SLIDE_TIME: Duration = Duration::from_secs(15);

/// The shown slide of `count`, moving on by itself; `go` shows another one
/// and starts the wait again.
fn slideshow(count: usize) -> (RwSignal<usize>, impl Fn(usize) + Copy + Send + Sync + 'static) {
    let current = RwSignal::new(0_usize);
    let timer = StoredValue::new(None::<IntervalHandle>);
    let restart = move || {
        if let Some(handle) = timer.get_value() {
            handle.clear();
        }
        let next = move || current.update(|index| *index = (*index + 1) % count.max(1));
        timer.set_value(set_interval_with_handle(next, SLIDE_TIME).ok());
    };
    Effect::new(restart);
    on_cleanup(move || {
        if let Some(handle) = timer.get_value() {
            handle.clear();
        }
    });
    let go = move |index: usize| {
        current.set(index % count.max(1));
        restart();
    };
    (current, go)
}

/// A few photos with arrows (Služby, the Sortiment pages). `class` is added
/// to the wrapper.
#[island]
pub fn PhotoCarousel(images: Vec<String>, alt: String, class: String, previous: String, next: String) -> impl IntoView {
    let count = images.len();
    let (current, go) = slideshow(count);
    view! {
        <div class=format!("sluzba-carousel-wrapper {class}").trim().to_string()>
            <button class="carousel-btn prev-btn" aria-label=previous on:click=move |_| go(current.get_untracked() + count - 1)>
                <svg viewBox="0 0 24 24" fill="white">
                    <path d="M15.41 16.59L10.83 12l4.58-4.59L14 6l-6 6 6 6 1.41-1.41z"/>
                </svg>
            </button>
            <div class="carousel-track-container">
                <ul class="carousel-track">
                    {images
                        .into_iter()
                        .enumerate()
                        .map(|(index, src)| {
                            view! {
                                <li class="carousel-slide" class:active=move || current.get() == index>
                                    <img alt=alt.clone() src=src/>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </div>
            <button class="carousel-btn next-btn" aria-label=next on:click=move |_| go(current.get_untracked() + 1)>
                <svg viewBox="0 0 24 24" fill="white">
                    <path d="M8.59 16.59L13.17 12 8.59 7.41 10 6l6 6-6 6-1.41-1.41z"/>
                </svg>
            </button>
        </div>
    }
}

/// A "might interest you" slide on the home page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slide {
    pub title: String,
    /// Trusted HTML typed in the admin.
    pub content: String,
    pub image: Option<String>,
}

/// The words of the carousels' buttons.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CarouselTexts {
    pub previous: String,
    pub next: String,
    /// "Snímek" (the dots read "Snímek 2").
    pub slide: String,
}

/// The home page's "might interest you" slides, with dots under them.
#[island]
pub fn Slides(slides: Vec<Slide>, texts: CarouselTexts) -> impl IntoView {
    let count = slides.len();
    let (current, go) = slideshow(count);
    view! {
        <div class="carousel-wrapper" id="zajimatCarousel">
            <button class="carousel-btn prev-btn" id="carouselPrev" aria-label=texts.previous on:click=move |_| go(current.get_untracked() + count - 1)>
                <ChevronLeft/>
            </button>
            <div class="carousel-container">
                {slides
                    .into_iter()
                    .enumerate()
                    .map(|(index, slide)| {
                        view! {
                            <div class="carousel-slide" class:active=move || current.get() == index>
                                <img class="carousel-bg-img" alt=slide.title.clone() src=slide.image/>
                                <div class="carousel-overlay"></div>
                                <div class="carousel-content">
                                    <h3>{slide.title}</h3>
                                    <div class="carousel-text">
                                        <p inner_html=slide.content></p>
                                    </div>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
            <button class="carousel-btn next-btn" id="carouselNext" aria-label=texts.next on:click=move |_| go(current.get_untracked() + 1)>
                <ChevronRight/>
            </button>
        </div>
        <div class="carousel-dots">
            {(0..count)
                .map(|index| {
                    view! {
                        <button
                            class="carousel-dot"
                            class:active=move || current.get() == index
                            aria-label=format!("{} {}", texts.slide, index + 1)
                            on:click=move |_| go(index)
                        ></button>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// The carousels' arrows.
#[component]
pub fn ChevronLeft() -> impl IntoView {
    view! {
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 18 9 12 15 6"/>
        </svg>
    }
}

#[component]
pub fn ChevronRight() -> impl IntoView {
    view! {
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="9 18 15 12 9 6"/>
        </svg>
    }
}

/// The headline whose lines (split by `<br>`) each fit on one line: the
/// font shrinks just enough, but not below `min` pixels; below that the
/// lines wrap as usual. (Line lengths differ per language and the fonts per
/// system, so no fixed size would do.)
#[island]
pub fn FitLines(min: f64, children: Children) -> impl IntoView {
    let heading = NodeRef::<html::H1>::new();
    Effect::new(move || {
        if let Some(heading) = heading.get() {
            super::browser::fit_lines(heading.into(), min);
        }
    });
    view! { <h1 node_ref=heading>{children()}</h1> }
}

/// A Google review as the carousel shows it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Review {
    pub name: String,
    /// The picture's URL; without one, a coloured initial.
    pub avatar: Option<String>,
    pub initial: String,
    pub color: String,
    /// "★★★★★"
    pub stars: String,
    /// "před 2 měsíci"
    pub time_ago: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReviewTexts {
    pub previous: String,
    pub next: String,
    /// "Recenze Google" under the name, with Google's logo.
    pub source: String,
    pub google_logo: String,
    /// "Užitečné"
    pub like: String,
}

/// The Google reviews, moving one card at a time with no end: once the
/// island is alive, a copy of all cards sits before and after the real
/// ones, and after each move past the real ones the track jumps back to
/// the same picture among them.
#[island]
pub fn ReviewsCarousel(reviews: Vec<Review>, texts: ReviewTexts) -> impl IntoView {
    let count = reviews.len();
    let track = NodeRef::<html::Div>::new();
    // The copies are there and the track is placed (in the browser only).
    let ready = RwSignal::new(false);
    // The first card shown, of the 3 × count; the real ones are count..2 × count.
    let first = RwSignal::new(count);
    let moving = RwSignal::new(false);
    // Bumped when the window is resized: the cards' width changes.
    let resized = RwSignal::new(0_u32);
    if count >= 2 {
        Effect::new(move || ready.set(true));
        let handle = window_event_listener(leptos::ev::resize, move |_| resized.update(|n| *n += 1));
        on_cleanup(move || handle.remove());
    }
    let step = move || {
        resized.track();
        track.get().map_or(0.0, |track| super::browser::card_step(&track))
    };
    let style = move || {
        ready.get().then(|| {
            let offset = first.get() as f64 * step();
            // While not moving, no transition: the jumps are instant.
            let transition = if moving.get() { "" } else { " transition: none;" };
            format!("transform: translateX(-{offset}px);{transition}")
        })
    };
    let go = move |by: isize| {
        if ready.get_untracked() && !moving.get_untracked() {
            moving.set(true);
            first.update(|first| *first = first.saturating_add_signed(by));
        }
    };
    let settle = move |event: leptos::ev::TransitionEvent| {
        // Only the track's own move (the cards may have transitions too).
        if event.target() != event.current_target() {
            return;
        }
        moving.set(false);
        first.update(|first| {
            if *first >= 2 * count {
                *first -= count;
            } else if *first < count {
                *first += count;
            }
        });
    };
    let reviews = StoredValue::new(reviews);
    let texts = StoredValue::new(texts);
    let cards = move |copy: bool| {
        let texts = texts.get_value();
        reviews
            .get_value()
            .into_iter()
            .map(|review| view! { <ReviewCard review=review texts=texts.clone() copy=copy/> })
            .collect_view()
    };
    let copies = move || ready.get().then(|| cards(true));
    view! {
        <div class="google-reviews-carousel">
            <button class="carousel-btn prev-btn" id="googleReviewsPrev" aria-label=texts.get_value().previous on:click=move |_| go(-1)>
                <ChevronLeft/>
            </button>
            <div class="google-reviews-viewport">
                <div class="google-reviews-grid" id="googleReviewsTrack" node_ref=track style=style on:transitionend=settle>
                    {copies}
                    {cards(false)}
                    {copies}
                </div>
            </div>
            <button class="carousel-btn next-btn" id="googleReviewsNext" aria-label=texts.get_value().next on:click=move |_| go(1)>
                <ChevronRight/>
            </button>
        </div>
    }
}

/// `copy`: one of the copies around the real cards, hidden from screen readers.
#[component]
fn ReviewCard(review: Review, texts: ReviewTexts, copy: bool) -> impl IntoView {
    let avatar = match review.avatar {
        Some(src) => view! { <img alt=review.name.clone() class="google-review-avatar-img" src=src/> }.into_any(),
        None => view! {
            <div class="google-review-avatar" style=format!("background-color: {}", review.color)>{review.initial}</div>
        }
        .into_any(),
    };
    view! {
        <div class="google-review-card" aria-hidden=copy.then_some("true")>
            <div class="google-review-card-header">
                {avatar}
                <div>
                    <div class="google-review-name">{review.name}</div>
                    <div class="google-review-source">
                        <img alt="Google" class="google-review-source-icon" src=texts.google_logo/>
                        " "
                        {texts.source}
                    </div>
                </div>
            </div>
            <div class="google-review-meta">
                <span class="google-review-stars">{review.stars}</span>
                " "
                <span class="google-review-time">{review.time_ago}</span>
            </div>
            <p class="google-review-text">{review.text}</p>
            <div class="google-review-like">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M1 21h4V9H1v12zm22-11c0-1.1-.9-2-2-2h-6.31l.95-4.57.03-.32c0-.41-.17-.79-.44-1.06L14.17 1 7.59 7.59C7.22 7.95 7 8.45 7 9v10c0 1.1.9 2 2 2h9c.83 0 1.54-.5 1.84-1.22l3.02-7.05c.09-.23.14-.47.14-.73v-2z"/>
                </svg>
                " "
                {texts.like}
            </div>
        </div>
    }
}

/// The buttons of a product filter: `options` are (value, label); "all"
/// shows everything. `class` is the row's.
#[island]
pub fn FilterButtons(class: String, options: Vec<(String, String)>) -> impl IntoView {
    let filter = super::shared::filter();
    view! {
        <div class=class>
            {options
                .into_iter()
                .enumerate()
                .map(|(index, (value, label))| {
                    let current = value.clone();
                    view! {
                        {(index > 0).then_some(" ")}
                        <button
                            type="button"
                            class="filter-btn"
                            class:active=move || filter.with(|filter| *filter == current)
                            on:click=move |_| filter.set(value.clone())
                        >
                            {label}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// Something the product filter shows for "all" and for the values in
/// `shown_for` (none: for "all" only); `children` is its content.
#[island]
pub fn Filtered(
    shown_for: Vec<String>,
    class: String,
    #[prop(optional)] id: Option<String>,
    children: Children,
) -> impl IntoView {
    let hidden = hidden_by_filter(shown_for);
    view! {
        <div id=id class=class style=move || hidden().then_some("display: none")>
            {children()}
        </div>
    }
}

/// A separator line the filter shows for "all" only.
#[island]
pub fn FilteredRule(class: String) -> impl IntoView {
    let hidden = hidden_by_filter(Vec::new());
    view! { <hr class=class style=move || hidden().then_some("display: none")/> }
}

fn hidden_by_filter(shown_for: Vec<String>) -> impl Fn() -> bool + Copy + Send + Sync + 'static {
    let filter = super::shared::filter();
    let shown_for = StoredValue::new(shown_for);
    move || filter.with(|filter| filter != "all" && !shown_for.with_value(|values| values.contains(filter)))
}
