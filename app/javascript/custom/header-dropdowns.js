// The language/currency and cart dropdowns in the header. A mouse still
// opens them on hover; a click, a tap or Enter/Space on the button opens
// them too, and a click elsewhere or Escape closes them. Listeners sit on
// the document because the cart widget is re-rendered by Turbo Streams.
const DROPDOWNS = ".lang-dropdown, .cart-dropdown"

const closeAll = (except) => {
    document.querySelectorAll(`${DROPDOWNS}`).forEach((dropdown) => {
        if (dropdown === except) return
        dropdown.classList.remove("open")
        dropdown.querySelector(":scope > button")?.setAttribute("aria-expanded", "false")
    })
}

document.addEventListener("click", (event) => {
    const button = event.target.closest(".lang-dropdown > button, .cart-dropdown > button")
    if (button) {
        const dropdown = button.parentElement
        const open = !dropdown.classList.contains("open")
        closeAll(dropdown)
        dropdown.classList.toggle("open", open)
        button.setAttribute("aria-expanded", String(open))
        return
    }

    if (!event.target.closest(DROPDOWNS)) closeAll()
})

document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return

    const dropdown = document.querySelector(`.lang-dropdown.open, .cart-dropdown.open`)
    if (!dropdown) return

    closeAll()
    dropdown.querySelector(":scope > button")?.focus()
})

// Leaving the dropdown with Tab closes it.
document.addEventListener("focusin", (event) => {
    if (!event.target.closest(DROPDOWNS)) closeAll()
})

document.addEventListener("turbo:before-cache", () => closeAll())
