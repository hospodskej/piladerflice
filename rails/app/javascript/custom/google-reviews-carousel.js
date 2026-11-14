document.addEventListener("turbo:load", function() {
    const track = document.getElementById("googleReviewsTrack");
    const prevBtn = document.getElementById("googleReviewsPrev");
    const nextBtn = document.getElementById("googleReviewsNext");

    if (!track || !prevBtn || !nextBtn) return;

    const realCards = Array.from(track.children);
    if (realCards.length < 2) return;

    let animating = false;

    function cardStep() {
        const gap = parseFloat(getComputedStyle(track).columnGap || 0);
        return realCards[0].getBoundingClientRect().width + gap;
    }

    // Keeps one cloned "preview" card past each edge, already fully rendered,
    // so the incoming card slides into view instead of popping in after the fact.
    function syncClones() {
        track.querySelectorAll("[data-clone]").forEach((el) => el.remove());

        const firstClone = realCards[0].cloneNode(true);
        firstClone.setAttribute("data-clone", "");
        firstClone.setAttribute("aria-hidden", "true");

        const lastClone = realCards[realCards.length - 1].cloneNode(true);
        lastClone.setAttribute("data-clone", "");
        lastClone.setAttribute("aria-hidden", "true");

        track.insertBefore(lastClone, track.firstChild);
        track.appendChild(firstClone);
    }

    function settleAtRest() {
        track.style.transition = "none";
        track.style.transform = `translateX(-${cardStep()}px)`;
        track.getBoundingClientRect();
        track.style.transition = "";
    }

    function rebuildTrack(newOrder) {
        track.querySelectorAll("[data-clone]").forEach((el) => el.remove());
        newOrder.forEach((card) => track.appendChild(card));
        syncClones();
        settleAtRest();
    }

    syncClones();
    settleAtRest();

    nextBtn.addEventListener("click", () => {
        if (animating) return;
        animating = true;

        track.style.transition = "transform 0.4s ease";
        track.style.transform = `translateX(-${2 * cardStep()}px)`;

        track.addEventListener("transitionend", function handler() {
            track.removeEventListener("transitionend", handler);
            realCards.push(realCards.shift());
            rebuildTrack(realCards);
            animating = false;
        }, { once: true });
    });

    prevBtn.addEventListener("click", () => {
        if (animating) return;
        animating = true;

        track.style.transition = "transform 0.4s ease";
        track.style.transform = "translateX(0)";

        track.addEventListener("transitionend", function handler() {
            track.removeEventListener("transitionend", handler);
            realCards.unshift(realCards.pop());
            rebuildTrack(realCards);
            animating = false;
        }, { once: true });
    });
});
