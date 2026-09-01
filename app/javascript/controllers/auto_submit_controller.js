import { Controller } from "@hotwired/stimulus"

// Submits the controller's form as soon as its value changes, so the cart
// quantity input doesn't need a separate "update" button.
export default class extends Controller {
  submit() {
    this.element.requestSubmit()
  }
}
