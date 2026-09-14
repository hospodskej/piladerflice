document.addEventListener("turbo:load", function() {
    const carousels = document.querySelectorAll("#zajimatCarousel, .sluzba-carousel-wrapper");

    if (carousels.length === 0) return;

    carousels.forEach(function(carousel) {

        const slides = carousel.querySelectorAll(".carousel-slide");

        const nextBtn = carousel.querySelector(".next-btn") || carousel.querySelector("#carouselNext");
        const prevBtn = carousel.querySelector(".prev-btn") || carousel.querySelector("#carouselPrev");
        const dotsContainer = carousel.parentElement?.querySelector(".carousel-dots");
        const dots = dotsContainer ? dotsContainer.querySelectorAll(".carousel-dot") : [];

        if (slides.length === 0 || !nextBtn || !prevBtn) return;

        let currentIndex = 0;
        let timer;
        const intervalTime = 15000;

        function goToSlide(index) {
            slides.forEach(slide => slide.classList.remove("active"));
            slides[index].classList.add("active");
            dots.forEach(dot => dot.classList.remove("active"));
            if (dots[index]) dots[index].classList.add("active");
            currentIndex = index;
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

        dots.forEach((dot, index) => {
            dot.addEventListener("click", () => {
                goToSlide(index);
                resetTimer();
            });
        });

        goToSlide(currentIndex);
        resetTimer();
    });
});
