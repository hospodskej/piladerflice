//! The Google reviews carousel.

use leptos::prelude::*;

use super::Accented;
use crate::islands::widgets::{self, ReviewTexts, ReviewsCarousel};
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
    let reviews = reviews
        .into_iter()
        .map(|review| widgets::Review {
            name: review.name,
            avatar: review.avatar.as_deref().map(asset),
            initial: review.initial,
            color: review.color.to_string(),
            stars: review.stars,
            time_ago: review.time_ago,
            text: review.text,
        })
        .collect();
    let texts = ReviewTexts {
        previous: t("common.previous"),
        next: t("common.next"),
        source: t("home.index.google_review_source"),
        google_logo: asset("google-review.webp"),
        like: t("home.index.google_review_like"),
    };
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
            <ReviewsCarousel reviews=reviews texts=texts/>
        </section>
    }
}
