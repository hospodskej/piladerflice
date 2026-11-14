import { Controller } from "@hotwired/stimulus"

// Suggestions while typing a town or an address, like Mapy.cz: a list under
// the field, picked with the mouse, a tap or the keyboard (arrows, Enter,
// Escape). "place" fills the town (and can send the form right away);
// "address" fills street, town and postcode. Suggestions come from
// /naseptavac-adres (AddressSuggestionsController).
export default class extends Controller {
  static targets = ["input"]
  static values = {
    url: String,
    kind: { type: String, default: "address" },
    country: String,       // CSS selector of the country <select>
    city: String,          // CSS selectors of the fields an address fills
    zip: String,
    submitOnPick: Boolean
  }

  connect() {
    this.list = document.createElement("ul")
    this.list.className = "address-suggestions"
    this.list.id = `${this.inputTarget.id || "address"}-suggestions`
    this.list.setAttribute("role", "listbox")
    this.list.hidden = true
    this.inputTarget.insertAdjacentElement("afterend", this.list)
    this.inputTarget.parentElement.classList.add("address-autocomplete")

    this.inputTarget.setAttribute("role", "combobox")
    this.inputTarget.setAttribute("aria-autocomplete", "list")
    this.inputTarget.setAttribute("aria-controls", this.list.id)
    this.inputTarget.setAttribute("aria-expanded", "false")
    this.inputTarget.setAttribute("autocomplete", "off")

    this.onInput = this.onInput.bind(this)
    this.onKeydown = this.onKeydown.bind(this)
    this.onBlur = () => setTimeout(() => this.close(), 150)
    this.inputTarget.addEventListener("input", this.onInput)
    this.inputTarget.addEventListener("keydown", this.onKeydown)
    this.inputTarget.addEventListener("blur", this.onBlur)
  }

  disconnect() {
    this.inputTarget.removeEventListener("input", this.onInput)
    this.inputTarget.removeEventListener("keydown", this.onKeydown)
    this.inputTarget.removeEventListener("blur", this.onBlur)
    this.request?.abort()
    clearTimeout(this.timer)
    this.list.remove()
  }

  onInput() {
    clearTimeout(this.timer)
    const text = this.inputTarget.value.trim()
    if (text.length < 3) return this.close()

    this.timer = setTimeout(() => this.fetchSuggestions(text), 250)
  }

  async fetchSuggestions(text) {
    this.request?.abort()
    this.request = new AbortController()
    const params = new URLSearchParams({ q: text, kind: this.kindValue, country: this.country() })
    try {
      const response = await fetch(`${this.urlValue}?${params}`, { signal: this.request.signal, headers: { Accept: "application/json" } })
      if (!response.ok) return this.close()
      this.show(await response.json())
    } catch (error) {
      if (error.name !== "AbortError") this.close()
    }
  }

  show(suggestions) {
    this.suggestions = suggestions
    this.active = -1
    this.list.replaceChildren(...suggestions.map((suggestion, index) => {
      const item = document.createElement("li")
      item.id = `${this.list.id}-${index}`
      item.setAttribute("role", "option")
      item.textContent = suggestion.label
      // mousedown, not click: it comes before the field's blur closes the list.
      item.addEventListener("mousedown", (event) => { event.preventDefault(); this.pick(index) })
      return item
    }))
    const open = suggestions.length > 0
    this.list.hidden = !open
    this.inputTarget.setAttribute("aria-expanded", String(open))
    this.inputTarget.removeAttribute("aria-activedescendant")
  }

  onKeydown(event) {
    if (this.list.hidden) return

    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault()
      const count = this.suggestions.length
      this.active = (this.active + (event.key === "ArrowDown" ? 1 : -1) + count) % count
      this.list.querySelectorAll("li").forEach((item, index) => item.setAttribute("aria-selected", String(index === this.active)))
      this.inputTarget.setAttribute("aria-activedescendant", `${this.list.id}-${this.active}`)
    } else if (event.key === "Enter" && this.active >= 0) {
      event.preventDefault()
      this.pick(this.active)
    } else if (event.key === "Escape") {
      this.close()
    }
  }

  pick(index) {
    const suggestion = this.suggestions[index]
    this.close()
    if (this.kindValue === "place") {
      this.inputTarget.value = suggestion.city
    } else {
      this.inputTarget.value = suggestion.street || this.inputTarget.value
      this.fill(this.cityValue, suggestion.city)
      this.fill(this.zipValue, suggestion.zip)
    }
    this.fill(this.countryValue, suggestion.country)
    if (this.submitOnPickValue) this.inputTarget.form?.requestSubmit()
  }

  fill(selector, value) {
    const field = selector && document.querySelector(selector)
    if (field && value) field.value = value
  }

  country() {
    const field = this.countryValue && document.querySelector(this.countryValue)
    return field?.value || "cz"
  }

  close() {
    this.list.hidden = true
    this.inputTarget.setAttribute("aria-expanded", "false")
    this.inputTarget.removeAttribute("aria-activedescendant")
  }
}
