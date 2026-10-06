import { Controller } from "@hotwired/stimulus"

// Shrinks the font just enough that each <br>-separated line fits without
// wrapping. Line lengths differ per locale and the system font stack renders
// differently per OS, so no fixed CSS font-size can guarantee this. If fitting
// would need a size below `min` (px), the lines wrap normally instead.
export default class extends Controller {
  static values = { min: Number }

  connect() {
    // Containment keeps the unwrapped lines from widening the parent (e.g. a
    // 1fr grid column sizes itself to fit its content's min-content width).
    this.element.style.contain = "inline-size"
    this.width = null
    this.observer = new ResizeObserver(([entry]) => {
      if (entry.contentRect.width === this.width) return
      this.width = entry.contentRect.width
      this.fit()
    })
    this.observer.observe(this.element)
    document.fonts.ready.then(() => this.fit())
  }

  disconnect() {
    this.observer.disconnect()
    this.element.style.contain = ""
    this.element.style.whiteSpace = ""
    this.element.style.fontSize = ""
  }

  fit() {
    const el = this.element
    el.style.fontSize = ""
    el.style.whiteSpace = "nowrap"
    const available = el.clientWidth
    if (el.scrollWidth <= available) return

    let size = parseFloat(getComputedStyle(el).fontSize) * available / el.scrollWidth
    if (size < this.minValue) {
      el.style.whiteSpace = ""
      return
    }
    el.style.fontSize = `${size}px`
    while (el.scrollWidth > available && size > 1) {
      size -= 0.5
      el.style.fontSize = `${size}px`
    }
  }
}
