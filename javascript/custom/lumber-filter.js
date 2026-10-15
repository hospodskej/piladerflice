document.addEventListener("click", function(event) {
    const btn = event.target.closest(".dimension-filter .filter-btn");
    if (!btn) return;

    const filterRow = btn.closest(".dimension-filter");
    const grid = document.getElementById(filterRow.dataset.gridTarget);
    if (!grid) return;

    filterRow.querySelectorAll(".filter-btn").forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");

    const filter = btn.dataset.filter;
    grid.querySelectorAll(".inventory-card").forEach((card) => {
        const matches = filter === "all" || card.dataset.filterValue === filter;
        card.style.display = matches ? "" : "none";
    });
});
