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

    function nextFrame(fn) {
        requestAnimationFrame(() => requestAnimationFrame(fn));
    }

    function onTransitionEnd(handler) {
        track.addEventListener("transitionend", handler, { once: true });
    }

    nextBtn.addEventListener("click", () => {
        if (animating) return;
        animating = true;

        track.style.transition = "transform 0.4s ease";
        track.style.transform = `translateX(-${cardStep()}px)`;

        onTransitionEnd(() => {
            track.style.transition = "none";
            track.appendChild(track.firstElementChild);
            track.style.transform = "translateX(0)";

            nextFrame(() => {
                track.style.transition = "";
                animating = false;
            });
        });
    });

    prevBtn.addEventListener("click", () => {
        if (animating) return;
        animating = true;

        track.style.transition = "none";
        track.insertBefore(track.lastElementChild, track.firstElementChild);
        track.style.transform = `translateX(-${cardStep()}px)`;

        nextFrame(() => {
            track.style.transition = "transform 0.4s ease";
            track.style.transform = "translateX(0)";
        });

        onTransitionEnd(() => {
            animating = false;
        });
    });
});
