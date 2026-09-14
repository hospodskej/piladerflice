document.addEventListener("turbo:load", function() {
    const track = document.getElementById("googleReviewsTrack");
    const prevBtn = document.getElementById("googleReviewsPrev");
    const nextBtn = document.getElementById("googleReviewsNext");

    if (!track || !prevBtn || !nextBtn) return;

    function scrollStep() {
        const card = track.querySelector(".google-review-card");
        return card ? card.offsetWidth + 20 : track.clientWidth;
    }

    nextBtn.addEventListener("click", () => {
        track.scrollBy({ left: scrollStep(), behavior: "smooth" });
    });

    prevBtn.addEventListener("click", () => {
        track.scrollBy({ left: -scrollStep(), behavior: "smooth" });
    });
});
