import { Controller } from "@hotwired/stimulus"

// The "Filtr" panel on a product page. Shows the cards matching the ticked
// values (any value within a group, all groups together; an empty group
// doesn't filter). A checkbox named "length" checks a card's data-length.
// On the firewood price grid it also hides the columns, rows and sections
// left with nothing in them; a plain card grid just closes up.
export default class extends Controller {
  static targets = ["grid", "section", "separator", "empty"]

  apply(event) {
    event?.preventDefault()
    const chosen = {}
    this.element.querySelectorAll(".variant-filter input[type=checkbox]:checked").forEach((input) => {
      (chosen[input.name] ||= new Set()).add(input.value)
    })
    const matches = (card) => Object.entries(chosen).every(([name, values]) => values.has(card.dataset[name]))

    let shownTotal = 0
    this.gridTargets.forEach((grid) => {
      if (!grid.querySelector(".matrix-cell")) {
        grid.querySelectorAll(".inventory-card").forEach((card) => {
          const match = matches(card)
          card.style.display = match ? "" : "none"
          if (match) shownTotal++
        })
        return
      }

      const rows = new Set(), columns = new Set()
      grid.querySelectorAll(".inventory-card").forEach((card) => {
        const match = matches(card)
        card.style.visibility = match ? "" : "hidden"
        if (match) {
          const cell = card.closest(".matrix-cell")
          rows.add(cell.dataset.row)
          columns.add(cell.dataset.length)
          shownTotal++
        }
      })

      // A cell stays (maybe empty, to keep the grid lined up) while both its
      // row and its column still have something to show.
      grid.querySelectorAll(".matrix-cell").forEach((cell) => {
        const { row, length } = cell.dataset
        const keep = cell.hasAttribute("data-corner") ? columns.size > 0 :
          (row === undefined || rows.has(row)) && (length === undefined || columns.has(length))
        cell.style.display = keep ? "" : "none"
      })
      // Columns stay card-sized: one length left shouldn't fill the page.
      grid.style.gridTemplateColumns = columns.size ? `90px repeat(${columns.size}, minmax(0, 260px))` : ""

      const section = this.sectionTargets.find((s) => s.contains(grid))
      if (section) section.style.display = columns.size === 0 ? "none" : ""
    })

    const shownSections = this.sectionTargets.filter((s) => s.style.display !== "none").length
    this.separatorTargets.forEach((hr) => { hr.style.display = shownSections < 2 ? "none" : "" })
    if (this.hasEmptyTarget) this.emptyTarget.hidden = shownTotal > 0
  }

  reset() {
    this.element.querySelectorAll(".variant-filter input[type=checkbox]").forEach((input) => { input.checked = false })
    this.apply()
  }
}
