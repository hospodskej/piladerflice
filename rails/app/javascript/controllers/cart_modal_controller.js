import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["dialog"]

  backdropClick(event) {
    if (!this.dialogTarget.contains(event.target)) {
      this.close()
    }
  }

  keydown(event) {
    if (event.key === "Escape") {
      this.close()
    }
  }

  close() {
    this.element.remove()
  }
}
