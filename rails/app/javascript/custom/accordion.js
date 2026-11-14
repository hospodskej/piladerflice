document.addEventListener("turbo:load", function() {
    const accordionHeaders = document.querySelectorAll(".accordion-header");

    accordionHeaders.forEach(header => {
        header.addEventListener("click", function() {
            const currentItem = this.parentElement;
            currentItem.classList.toggle("active");
        });
    });
});