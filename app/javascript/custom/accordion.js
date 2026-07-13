document.addEventListener("turbo:load", function() {
    const accordionHeaders = document.querySelectorAll(".accordion-header");

    accordionHeaders.forEach(header => {
        header.addEventListener("click", function() {
            const currentItem = this.parentElement;

            // Toggle the active class on the one we just clicked
            currentItem.classList.toggle("active");
        });
    });
});