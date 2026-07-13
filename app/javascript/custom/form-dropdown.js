document.addEventListener("DOMContentLoaded", () => {
    // Find all standard selects inside the form
    const selects = document.querySelectorAll(".kalkulace-grid-4 select");

    selects.forEach(originalSelect => {
        // 1. Hide the original select
        originalSelect.style.display = 'none';

        // 2. Create the main wrapper
        const wrapper = document.createElement('div');
        wrapper.className = 'custom-select-wrapper';
        originalSelect.parentNode.insertBefore(wrapper, originalSelect);
        wrapper.appendChild(originalSelect);

        // 3. Create the visible display box
        const display = document.createElement('div');
        display.className = 'custom-select-display';
        wrapper.appendChild(display);

        // 4. Create the dropdown menu
        const menu = document.createElement('div');
        menu.className = 'custom-select-menu';
        wrapper.appendChild(menu);

        // MAGIC FORMATTER: Splits the text by space. Makes first word blue, rest black.
        const formatText = (text) => {
            const parts = text.trim().split(' ');
            if (parts.length > 1) {
                const first = parts.shift();
                return `<span style="color: #2C8FFF;">${first}</span> <span>${parts.join(' ')}</span>`;
            }
            return `<span style="color: #2C8FFF;">${text}</span>`;
        };

        // 5. Build the menu items based on the original HTML options
        Array.from(originalSelect.options).forEach((opt, index) => {
            const item = document.createElement('div');
            item.className = 'custom-select-item';
            item.innerHTML = formatText(opt.text);

            // When an item is clicked, update the display and the hidden native select
            item.addEventListener('click', (e) => {
                e.stopPropagation();
                originalSelect.selectedIndex = index;
                display.innerHTML = formatText(opt.text);
                menu.classList.remove('open');
                display.classList.remove('open');
            });
            menu.appendChild(item);
        });

        // Set initial display value
        display.innerHTML = formatText(originalSelect.options[originalSelect.selectedIndex].text);

        // 6. Toggle menu open/close on click
        display.addEventListener('click', (e) => {
            e.stopPropagation();
            // Close any other open menus first
            document.querySelectorAll('.custom-select-menu').forEach(m => {
                if (m !== menu) m.classList.remove('open');
            });
            document.querySelectorAll('.custom-select-display').forEach(d => {
                if (d !== display) d.classList.remove('open');
            });

            menu.classList.toggle('open');
            display.classList.toggle('open');
        });
    });

    // 7. Close menus if user clicks anywhere outside
    document.addEventListener('click', () => {
        document.querySelectorAll('.custom-select-menu').forEach(m => m.classList.remove('open'));
        document.querySelectorAll('.custom-select-display').forEach(d => d.classList.remove('open'));
    });
});