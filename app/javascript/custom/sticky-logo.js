const initStickyLogo = () => {
  const bar = document.getElementById("stickyLogoBar");
  const header = document.querySelector(".site-header");
  if (!bar || !header) return;

  const updateVisibility = () => {
    bar.classList.toggle("is-visible", window.scrollY > header.offsetHeight);
  };

  updateVisibility();
  window.addEventListener("scroll", updateVisibility, { passive: true });
  window.addEventListener("resize", updateVisibility);
};

document.addEventListener("DOMContentLoaded", initStickyLogo);
document.addEventListener("turbo:load", initStickyLogo);
