// Wrap everything in a function so it can run on Rails navigations
const initFormLogic = () => {
    // --- 1. TOGGLE BUTTONS LOGIC ---
    const toggleBtns = document.querySelectorAll(".toggle-btn");
    toggleBtns.forEach(btn => {
        // Using onclick instead of addEventListener prevents duplicate listeners if Rails loads this twice
        btn.onclick = function() {
            toggleBtns.forEach(b => b.classList.remove("active"));
            this.classList.add("active");
        };
    });

    // --- 2. DYNAMIC ROWS LOGIC ---
    const container = document.getElementById('kalkulace-items-container');
    const addBtn = document.getElementById('add-kalkulace-row');

    if (!container || !addBtn) return; // Safety check if we are not on the Kontakt page

    // Initial setup: apply dropdowns to the first row
    container.querySelectorAll('.kalkulace-row').forEach(row => initializeRow(row));

    // Add Button Logic
    addBtn.onclick = (e) => {
        e.preventDefault(); // Prevent form jumping

        const template = container.querySelector('.kalkulace-row');
        const newRow = template.cloneNode(true);

        // Extract the native <select> BEFORE destroying the old cloned wrappers
        newRow.querySelectorAll('.custom-select-wrapper').forEach(wrapper => {
            const nativeSelect = wrapper.querySelector('select');
            if (nativeSelect) {
                // Move the select back out of the wrapper
                wrapper.parentNode.insertBefore(nativeSelect, wrapper);
            }
            // Now it is safe to delete the cloned visual wrapper
            wrapper.remove();
        });

        // Reset values and ensure native select is ready
        newRow.querySelectorAll('select').forEach(s => {
            s.selectedIndex = 0;
            s.style.display = 'block';
        });

        container.appendChild(newRow);

        // Initialize the new row
        initializeRow(newRow);
    };
};

// Helper: Initializes a row (dropdowns + delete button)
function initializeRow(row) {
    row.querySelectorAll('select').forEach(select => {
        // Prevent double-rendering
        if (select.closest('.custom-select-wrapper')) return;

        createCustomDropdown(select);
    });

    const delBtn = row.querySelector('.delete-item-btn');
    if (delBtn) {
        delBtn.onclick = (e) => {
            e.preventDefault();
            row.remove();
        };
    }
}

// Helper: Transforms a single select into the custom dual-color dropdown
function createCustomDropdown(select) {
    select.style.display = 'none';

    const wrapper = document.createElement('div');
    wrapper.className = 'custom-select-wrapper';
    select.parentNode.insertBefore(wrapper, select);
    wrapper.appendChild(select); // Select is safely hidden inside the wrapper

    const display = document.createElement('div');
    display.className = 'custom-select-display';
    wrapper.appendChild(display);

    const menu = document.createElement('div');
    menu.className = 'custom-select-menu';
    wrapper.appendChild(menu);

    // Formatter for Blue/Black text
    const formatText = (text) => {
        const parts = text.trim().split(' ');
        if (parts.length > 1) {
            const first = parts.shift();
            return `<span style="color: #2C8FFF;">${first}</span> <span style="color: #000000;">${parts.join(' ')}</span>`;
        }
        return `<span style="color: #2C8FFF;">${text}</span>`;
    };

    Array.from(select.options).forEach((opt, index) => {
        const item = document.createElement('div');
        item.className = 'custom-select-item';
        item.innerHTML = formatText(opt.text);
        item.onclick = (e) => {
            e.stopPropagation();
            select.selectedIndex = index;
            display.innerHTML = formatText(opt.text);
            menu.classList.remove('open');
            display.classList.remove('open');
        };
        menu.appendChild(item);
    });

    display.innerHTML = formatText(select.options[select.selectedIndex].text);
    display.onclick = (e) => {
        e.stopPropagation();
        document.querySelectorAll('.custom-select-menu').forEach(m => {
            if (m !== menu) m.classList.remove('open');
        });
        menu.classList.toggle('open');
        display.classList.toggle('open');
    };
}

// Close menus on outside click (only needs to be attached once globally)
document.addEventListener('click', () => {
    document.querySelectorAll('.custom-select-menu').forEach(m => m.classList.remove('open'));
    document.querySelectorAll('.custom-select-display').forEach(d => d.classList.remove('open'));
});

// --- THE RAILS FIX ---
// Listen for standard browser loads
document.addEventListener("DOMContentLoaded", initFormLogic);
// Listen for modern Rails Turbo loads
document.addEventListener("turbo:load", initFormLogic);
// Listen for older Rails Turbolinks loads (just in case)
document.addEventListener("turbolinks:load", initFormLogic);