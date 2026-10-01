import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["button", "menu", "input", "flag", "label"]

  toggle(event) {
    event.stopPropagation()

    if (this.menuTarget.classList.contains("open")) {
      this.close()
      return
    }

    const rect = this.buttonTarget.getBoundingClientRect()
    this.menuTarget.style.position = "fixed"
    this.menuTarget.style.top = `${rect.bottom + 4}px`
    this.menuTarget.style.left = `${rect.left}px`
    this.menuTarget.style.minWidth = `${rect.width}px`
    this.menuTarget.classList.add("open")
    this.buttonTarget.classList.add("open")
  }

  select(event) {
    event.stopPropagation()
    const option = event.currentTarget
    this.inputTarget.value = option.dataset.value
    this.labelTarget.textContent = option.dataset.value
    this.flagTarget.innerHTML = option.querySelector("svg").outerHTML
    this.close()
  }

  closeIfOutside(event) {
    if (!this.element.contains(event.target)) {
      this.close()
    }
  }

  close() {
    this.menuTarget.classList.remove("open")
    this.buttonTarget.classList.remove("open")
  }
}
