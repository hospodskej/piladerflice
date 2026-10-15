document.addEventListener("turbo:load", function() {
    const checkbox = document.getElementById("termsAgreement");
    const error = document.getElementById("termsError");

    if (!checkbox || !error) return;

    const label = checkbox.closest(".custom-checkbox-label");
    const form = checkbox.closest("form");

    form.addEventListener("submit", (event) => {
        if (!checkbox.checked) {
            event.preventDefault();
            error.hidden = false;
            label.classList.add("checkout-checkbox-invalid");
            error.scrollIntoView({ behavior: "smooth", block: "center" });
        }
    });

    checkbox.addEventListener("change", () => {
        if (checkbox.checked) {
            error.hidden = true;
            label.classList.remove("checkout-checkbox-invalid");
        }
    });
});
