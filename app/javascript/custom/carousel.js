document.addEventListener("turbo:load", function() {
    // 1. Find ALL carousels on the current page (Homepage or Služby page)
    const carousels = document.querySelectorAll("#zajimatCarousel, .sluzba-carousel-wrapper");

    // If there are no carousels on this page, stop.
    if (carousels.length === 0) return;

    // 2. Loop through each carousel and give it its own independent logic
    carousels.forEach(function(carousel) {

        const slides = carousel.querySelectorAll(".carousel-slide");

        // Find the buttons specifically INSIDE this current carousel
        const nextBtn = carousel.querySelector(".next-btn") || carousel.querySelector("#carouselNext");
        const prevBtn = carousel.querySelector(".prev-btn") || carousel.querySelector("#carouselPrev");

        // Safety check: if this specific carousel is missing parts, skip it
        if (slides.length === 0 || !nextBtn || !prevBtn) return;

        let currentIndex = 0;
        let timer;
        const intervalTime = 15000;

        function goToSlide(index) {
            slides.forEach(slide => slide.classList.remove("active"));
            slides[index].classList.add("active");
        }

        function nextSlide() {
            currentIndex = (currentIndex + 1) % slides.length;
            goToSlide(currentIndex);
        }

        function prevSlide() {
            currentIndex = (currentIndex - 1 + slides.length) % slides.length;
            goToSlide(currentIndex);
        }

        function resetTimer() {
            clearInterval(timer);
            timer = setInterval(nextSlide, intervalTime);
        }

        nextBtn.addEventListener("click", () => {
            nextSlide();
            resetTimer();
        });

        prevBtn.addEventListener("click", () => {
            prevSlide();
            resetTimer();
        });

        // Start this specific carousel
        goToSlide(currentIndex);
        resetTimer();
    });
});