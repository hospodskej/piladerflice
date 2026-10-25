import { Controller } from "@hotwired/stimulus"

// The firewood page's "Filtr" panel. Shows the cards matching the ticked
// variants, amounts and lengths (any value within a group, all groups
// together; an empty group doesn't filter), and hides the grid columns,
// rows and sections that are left with nothing in them.
export default class extends Controller {
  static targets = ["grid", "section", "separator", "empty"]

  apply(event) {
    event?.preventDefault()
    const chosen = (name) => new Set([...this.element.querySelectorAll(`input[name="${name}"]:checked`)].map((input) => input.value))
    const kinds = chosen("kind"), amounts = chosen("amount"), lengths = chosen("length")
    const pass = (set, value) => set.size === 0 || set.has(value)

    let shownTotal = 0
    this.gridTargets.forEach((grid) => {
      const rows = new Set(), columns = new Set()
      grid.querySelectorAll(".inventory-card").forEach((card) => {
        const match = pass(kinds, card.dataset.kind) && pass(amounts, card.dataset.amount) && pass(lengths, card.dataset.length)
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
      grid.style.gridTemplateColumns = columns.size ? `90px repeat(${columns.size}, minmax(0, 1fr))` : ""

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
