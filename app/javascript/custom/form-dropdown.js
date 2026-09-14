document.addEventListener("DOMContentLoaded", () => {
    const selects = document.querySelectorAll(".kalkulace-grid-4 select");

    selects.forEach(originalSelect => {
        originalSelect.style.display = 'none';

        const wrapper = document.createElement('div');
        wrapper.className = 'custom-select-wrapper';
        originalSelect.parentNode.insertBefore(wrapper, originalSelect);
        wrapper.appendChild(originalSelect);

        const display = document.createElement('div');
        display.className = 'custom-select-display';
        wrapper.appendChild(display);

        const menu = document.createElement('div');
        menu.className = 'custom-select-menu';
        wrapper.appendChild(menu);

        const formatText = (text) => {
            const parts = text.trim().split(' ');
            if (parts.length > 1) {
                const first = parts.shift();
                return `<span style="color: #2C8FFF;">${first}</span> <span>${parts.join(' ')}</span>`;
            }
            return `<span style="color: #2C8FFF;">${text}</span>`;
        };

        Array.from(originalSelect.options).forEach((opt, index) => {
            const item = document.createElement('div');
            item.className = 'custom-select-item';
            item.innerHTML = formatText(opt.text);

            item.addEventListener('click', (e) => {
                e.stopPropagation();
                originalSelect.selectedIndex = index;
                display.innerHTML = formatText(opt.text);
                menu.classList.remove('open');
                display.classList.remove('open');
            });
            menu.appendChild(item);
        });

        display.innerHTML = formatText(originalSelect.options[originalSelect.selectedIndex].text);

        display.addEventListener('click', (e) => {
            e.stopPropagation();
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

    document.addEventListener('click', () => {
        document.querySelectorAll('.custom-select-menu').forEach(m => m.classList.remove('open'));
        document.querySelectorAll('.custom-select-display').forEach(d => d.classList.remove('open'));
    });
});
