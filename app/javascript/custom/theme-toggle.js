function applyTheme(theme) {
  document.documentElement.classList.toggle("dark-mode", theme === "dark");
  try {
    localStorage.setItem("theme", theme);
  } catch (e) {}
}

document.addEventListener("click", (event) => {
  if (!event.target.closest("#themeToggle")) return;

  const isDark = document.documentElement.classList.contains("dark-mode");
  applyTheme(isDark ? "light" : "dark");
});
