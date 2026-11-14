// The dark/light switch. The choice is kept on this device (localStorage);
// for a signed-in customer it is also saved to the account, so signing in
// on another device brings the same mode along. The layout applies it
// before the first paint; this keeps it in step after Turbo navigations
// (e.g. right after signing in) and saves changes.

function preference() {
  return document.querySelector('meta[name="theme-preference"]');
}

function applyTheme(theme) {
  document.documentElement.classList.toggle("dark-mode", theme === "dark");
  try {
    localStorage.setItem("theme", theme);
  } catch (e) {}
}

function saveToAccount(theme) {
  const meta = preference();
  if (!meta) return;

  meta.content = theme;
  const token = document.querySelector('meta[name="csrf-token"]')?.content;
  fetch(meta.dataset.url, {
    method: "PATCH",
    headers: { "Content-Type": "application/json", "X-CSRF-Token": token, Accept: "application/json" },
    body: JSON.stringify({ theme }),
    credentials: "same-origin",
  }).catch(() => {});
}

function syncWithAccount() {
  const meta = preference();
  if (!meta) return;

  if (meta.content) {
    applyTheme(meta.content);
  } else {
    // Signed in for the first time since this existed: the account takes
    // this device's choice.
    let local = null;
    try {
      local = localStorage.getItem("theme");
    } catch (e) {}
    if (local) saveToAccount(local);
  }
}

document.addEventListener("turbo:load", syncWithAccount);

document.addEventListener("click", (event) => {
  if (!event.target.closest(".theme-toggle")) return;

  const theme = document.documentElement.classList.contains("dark-mode") ? "light" : "dark";
  applyTheme(theme);
  saveToAccount(theme);
});
