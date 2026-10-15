document.addEventListener("click", function(event) {
    const btn = event.target.closest(".qty-plus, .qty-minus");
    if (!btn) return;

    const row = btn.closest(".qty-cart-row");
    const input = row?.querySelector(".qty-value");
    if (!input) return;

    const min = parseInt(input.min || "1", 10);
    const current = parseInt(input.value, 10) || min;
    const next = btn.classList.contains("qty-plus") ? current + 1 : Math.max(min, current - 1);

    input.value = next;
});
