import { Controller } from "@hotwired/stimulus"

// Dismisses the "added to cart" confirmation modal (see
// shared/_cart_added_modal) on close-button click, backdrop click, or
// pressing Escape. The modal's root element *is* this controller's element
// (a full-screen backdrop), so closing just empties the persistent
// #cart-added-modal container it lives in.
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
