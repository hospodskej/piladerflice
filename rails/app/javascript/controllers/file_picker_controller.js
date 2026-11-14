import { Controller } from "@hotwired/stimulus"

// A styled replacement for the browser's native file input: the real input
// stays in the page (visually hidden, still focusable and submitted), and this
// shows the name of the chosen file next to the button.
export default class extends Controller {
  static targets = ["input", "name", "submit"]
  static values = { empty: String }

  connect() {
    this.update()
  }

  update() {
    const file = this.inputTarget.files[0]
    this.nameTarget.textContent = file ? file.name : this.emptyValue
    this.nameTarget.classList.toggle("has-file", Boolean(file))
    if (this.hasSubmitTarget) this.submitTarget.disabled = !file
  }
}
