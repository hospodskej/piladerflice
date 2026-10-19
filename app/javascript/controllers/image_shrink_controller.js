import { Controller } from "@hotwired/stimulus"

// Shrinks a photo in the browser before it is uploaded: at most 1600 px on
// its longer side, saved as WebP. The pages show product and promo photos at
// most ~730 px wide, so this stays sharp on retina screens, and a 5-10 MB
// phone photo becomes a few hundred KB. The server needs no image library for
// it. A picture the browser can't read is uploaded as it is.
export default class extends Controller {
  static values = { maxSize: { type: Number, default: 1600 }, quality: { type: Number, default: 0.82 } }

  connect() {
    this.waitForShrinking = this.waitForShrinking.bind(this)
    this.element.form?.addEventListener("submit", this.waitForShrinking)
  }

  disconnect() {
    this.element.form?.removeEventListener("submit", this.waitForShrinking)
  }

  shrink() {
    const file = this.element.files[0]
    if (!file || !["image/jpeg", "image/png", "image/webp"].includes(file.type)) return

    this.pending = this.shrunk(file)
      .then((smaller) => {
        if (smaller && this.element.files[0] === file) {
          const files = new DataTransfer()
          files.items.add(smaller)
          this.element.files = files.files
        }
      })
      .catch(() => {}) // upload the original
      .finally(() => { this.pending = null })
  }

  // Saving right after picking a photo waits for it to be shrunk.
  waitForShrinking(event) {
    if (!this.pending) return

    event.preventDefault()
    this.pending.then(() => this.element.form.requestSubmit(event.submitter))
  }

  async shrunk(file) {
    const bitmap = await createImageBitmap(file, { imageOrientation: "from-image" })
    const scale = Math.min(1, this.maxSizeValue / Math.max(bitmap.width, bitmap.height))
    if (scale === 1 && file.size <= 500_000) return null // already small enough

    const canvas = document.createElement("canvas")
    canvas.width = Math.round(bitmap.width * scale)
    canvas.height = Math.round(bitmap.height * scale)
    canvas.getContext("2d").drawImage(bitmap, 0, 0, canvas.width, canvas.height)
    bitmap.close()

    let blob = await this.encode(canvas, "image/webp")
    if (blob?.type !== "image/webp") blob = await this.encode(canvas, "image/jpeg") // Safari can't write WebP
    if (!blob || blob.size >= file.size) return null

    const name = file.name.replace(/\.[^.]*$/, "") + (blob.type === "image/webp" ? ".webp" : ".jpg")
    return new File([blob], name, { type: blob.type, lastModified: file.lastModified })
  }

  encode(canvas, type) {
    return new Promise((resolve) => canvas.toBlob(resolve, type, this.qualityValue))
  }
}
