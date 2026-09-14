document.addEventListener("turbo:load", function() {
    const track = document.getElementById("googleReviewsTrack");
    const prevBtn = document.getElementById("googleReviewsPrev");
    const nextBtn = document.getElementById("googleReviewsNext");

    if (!track || !prevBtn || !nextBtn) return;
    if (track.children.length < 2) return;

    let animating = false;

    function cardStep() {
        const card = track.firstElementChild;
        const gap = parseFloat(getComputedStyle(track).columnGap || 0);
        return card.getBoundingClientRect().width + gap;
    }

    function withoutTransition(fn) {
        track.style.transition = "none";
        fn();
        track.getBoundingClientRect();
        track.style.transition = "";
    }

    function onTransitionEnd(handler) {
        track.addEventListener("transitionend", handler, { once: true });
    }

    nextBtn.addEventListener("click", () => {
        if (animating) return;
        animating = true;

        track.style.transform = `translateX(-${cardStep()}px)`;
        onTransitionEnd(() => {
            withoutTransition(() => {
                track.appendChild(track.firstElementChild);
                track.style.transform = "translateX(0)";
            });
            animating = false;
        });
    });

    prevBtn.addEventListener("click", () => {
        if (animating) return;
        animating = true;

        withoutTransition(() => {
            track.insertBefore(track.lastElementChild, track.firstElementChild);
            track.style.transform = `translateX(-${cardStep()}px)`;
        });

        requestAnimationFrame(() => {
            track.style.transform = "translateX(0)";
        });

        onTransitionEnd(() => {
            animating = false;
        });
    });
});
