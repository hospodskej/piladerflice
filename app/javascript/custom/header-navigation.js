const initNavHighlight = () => {
    if (window.location.pathname === '/kontakt') {

        const cenikLink = document.getElementById('nav-cenik');
        const kontaktLink = document.getElementById('nav-kontakt');
        const activeClassName = 'active';

        const updateNavHighlight = () => {
            if (cenikLink) cenikLink.classList.remove(activeClassName);
            if (kontaktLink) kontaktLink.classList.remove(activeClassName);

            if (window.location.hash === '#cenik') {
                if (cenikLink) cenikLink.classList.add(activeClassName);
            } else {
                if (kontaktLink) kontaktLink.classList.add(activeClassName);
            }
        };

        updateNavHighlight();

        window.addEventListener('hashchange', updateNavHighlight);
    }
};

document.addEventListener("DOMContentLoaded", initNavHighlight);
document.addEventListener("turbo:load", initNavHighlight);
document.addEventListener("turbolinks:load", initNavHighlight);
