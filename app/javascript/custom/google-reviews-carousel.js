document.addEventListener("turbo:load", function() {
    const track = document.getElementById("googleReviewsTrack");
    const prevBtn = document.getElementById("googleReviewsPrev");
    const nextBtn = document.getElementById("googleReviewsNext");

    if (!track || !prevBtn || !nextBtn) return;

    function scrollStep() {
        const card = track.querySelector(".google-review-card");
        return card ? card.offsetWidth + 20 : track.clientWidth;
    }

    function maxScrollLeft() {
        return track.scrollWidth - track.clientWidth;
    }

    nextBtn.addEventListener("click", () => {
        if (track.scrollLeft >= maxScrollLeft() - 1) {
            track.scrollTo({ left: 0, behavior: "smooth" });
        } else {
            track.scrollBy({ left: scrollStep(), behavior: "smooth" });
        }
    });

    prevBtn.addEventListener("click", () => {
        if (track.scrollLeft <= 0) {
            track.scrollTo({ left: maxScrollLeft(), behavior: "smooth" });
        } else {
            track.scrollBy({ left: -scrollStep(), behavior: "smooth" });
        }
    });
});
