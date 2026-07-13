document.addEventListener("turbo:load", function() {
    const carousel = document.getElementById("zajimatCarousel");

    // If we aren't on a page with a carousel, stop running the script
    if (!carousel) return;

    const slides = carousel.querySelectorAll(".carousel-slide");
    const nextBtn = document.getElementById("carouselNext");
    const prevBtn = document.getElementById("carouselPrev");

    let currentIndex = 0;
    let timer;
    const intervalTime = 15000; // 15 seconds in milliseconds

    // Function to switch slides
    function goToSlide(index) {
        // Remove active class from all slides
        slides.forEach(slide => slide.classList.remove("active"));

        // Add active class to the target slide
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

    // The 15-second auto-rotate timer
    function resetTimer() {
        clearInterval(timer);
        timer = setInterval(nextSlide, intervalTime);
    }

    // Event Listeners for the arrows
    nextBtn.addEventListener("click", () => {
        nextSlide();
        resetTimer(); // Reset timer so it doesn't auto-skip immediately after clicking
    });

    prevBtn.addEventListener("click", () => {
        prevSlide();
        resetTimer();
    });

    goToSlide(currentIndex);

    // Start the timer when the page loads
    resetTimer();
});