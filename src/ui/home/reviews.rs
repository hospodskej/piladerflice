//! The Google reviews carousel.

use leptos::prelude::*;

use super::Accented;
use super::info::{ChevronLeft, ChevronRight};
use crate::ui::{asset, t};

pub struct Review {
    pub name: String,
    /// A logical asset name; without one, a coloured initial.
    pub avatar: Option<String>,
    pub initial: String,
    pub color: &'static str,
    /// "★★★★★"
    pub stars: String,
    /// "před 2 měsíci"
    pub time_ago: String,
    pub text: String,
}

#[component]
pub fn GoogleReviews(reviews: Vec<Review>) -> impl IntoView {
    view! {
        <section class="google-reviews-section">
            <h2><Accented rest="home.index.google_reviews_title_rest" accent="home.index.google_reviews_title_accent"/></h2>
            <div class="google-reviews-header">
                <div class="google-reviews-rating">
                    <img alt="Google" class="google-reviews-logo" src=asset("google-review.webp")/>
                    <div>
                        <div class="google-reviews-score">"4.6 "<span class="google-reviews-stars">"★★★★★"</span></div>
                        <div class="google-reviews-count">{t("home.index.google_reviews_count")}</div>
                    </div>
                </div>
                <div class="google-reviews-divider"></div>
                <div class="google-reviews-verified">
                    <span class="google-reviews-check">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="10"/>
                            <polyline points="8 12 11 15 16 9"/>
                        </svg>
                    </span>
                    " "
                    <div>
                        <div class="google-reviews-verified-title">{t("home.index.google_reviews_verified_title")}</div>
                        <div class="google-reviews-verified-sub">{t("home.index.google_reviews_verified_sub")}</div>
                    </div>
                </div>
            </div>
            <div class="google-reviews-carousel">
                <button class="carousel-btn prev-btn" id="googleReviewsPrev" aria-label=t("common.previous")>
                    <ChevronLeft/>
                </button>
                <div class="google-reviews-viewport">
                    <div class="google-reviews-grid" id="googleReviewsTrack">
                        {reviews.into_iter().map(|review| view! { <ReviewCard review=review/> }).collect_view()}
                    </div>
                </div>
                <button class="carousel-btn next-btn" id="googleReviewsNext" aria-label=t("common.next")>
                    <ChevronRight/>
                </button>
            </div>
        </section>
    }
}

#[component]
fn ReviewCard(review: Review) -> impl IntoView {
    let avatar = match review.avatar {
        Some(avatar) => {
            view! { <img alt=review.name.clone() class="google-review-avatar-img" src=asset(&avatar)/> }.into_any()
        }
        None => view! {
            <div class="google-review-avatar" style=format!("background-color: {}", review.color)>{review.initial}</div>
        }
        .into_any(),
    };
    view! {
        <div class="google-review-card">
            <div class="google-review-card-header">
                {avatar}
                <div>
                    <div class="google-review-name">{review.name}</div>
                    <div class="google-review-source">
                        <img alt="Google" class="google-review-source-icon" src=asset("google-review.webp")/>
                        " "
                        {t("home.index.google_review_source")}
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
                {t("home.index.google_review_like")}
            </div>
        </div>
    }
}
