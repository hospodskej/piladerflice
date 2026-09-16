const COOKIE_NAME = "cookie_consent";
const COOKIE_MAX_AGE_DAYS = 180;

function getConsentCookie() {
  const match = document.cookie.match(new RegExp(`(?:^|; )${COOKIE_NAME}=([^;]*)`));
  return match ? decodeURIComponent(match[1]) : null;
}

function setConsentCookie(value) {
  const maxAge = COOKIE_MAX_AGE_DAYS * 24 * 60 * 60;
  document.cookie = `${COOKIE_NAME}=${encodeURIComponent(value)}; path=/; max-age=${maxAge}; SameSite=Lax`;
}

function showBanner() {
  const banner = document.getElementById("cookie-consent-banner");
  if (banner) banner.hidden = false;
}

function hideBanner() {
  const banner = document.getElementById("cookie-consent-banner");
  if (banner) banner.hidden = true;
}

document.addEventListener("DOMContentLoaded", () => {
  if (!getConsentCookie()) showBanner();
});

document.addEventListener("click", (event) => {
  const choiceBtn = event.target.closest("[data-cookie-consent]");
  if (choiceBtn) {
    setConsentCookie(choiceBtn.dataset.cookieConsent);
    hideBanner();
    return;
  }

  if (event.target.closest("[data-cookie-settings]")) {
    event.preventDefault();
    showBanner();
  }
});
