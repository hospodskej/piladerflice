// Wrap our logic in a function so we can reuse it
const initNavHighlight = () => {
    // 1. Only run if we are on the Kontakt page
    if (window.location.pathname === '/kontakt') {

        const cenikLink = document.getElementById('nav-cenik');
        const kontaktLink = document.getElementById('nav-kontakt');
        const activeClassName = 'active'; // The class that makes it blue

        const updateNavHighlight = () => {
            // Strip the active class from both
            if (cenikLink) cenikLink.classList.remove(activeClassName);
            if (kontaktLink) kontaktLink.classList.remove(activeClassName);

            // Apply it to the correct one
            if (window.location.hash === '#cenik') {
                if (cenikLink) cenikLink.classList.add(activeClassName);
            } else {
                if (kontaktLink) kontaktLink.classList.add(activeClassName);
            }
        };

        // Run immediately
        updateNavHighlight();

        // Listen for anchor clicks (hash changes)
        window.addEventListener('hashchange', updateNavHighlight);
    }
};

// Listen for standard browser loads
document.addEventListener("DOMContentLoaded", initNavHighlight);
// Listen for modern Rails Turbo loads
document.addEventListener("turbo:load", initNavHighlight);
// Listen for older Rails Turbolinks loads (just in case)
document.addEventListener("turbolinks:load", initNavHighlight);