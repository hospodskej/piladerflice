const initFormLogic = () => {
    const toggleBtns = document.querySelectorAll(".toggle-btn");
    toggleBtns.forEach(btn => {
        btn.onclick = function() {
            toggleBtns.forEach(b => b.classList.remove("active"));
            this.classList.add("active");
        };
    });

    const container = document.getElementById('kalkulace-items-container');
    const addBtn = document.getElementById('add-kalkulace-row');

    if (!container || !addBtn) return;

    container.querySelectorAll('.kalkulace-row').forEach(row => initializeRow(row));

    addBtn.onclick = (e) => {
        e.preventDefault();

        const template = container.querySelector('.kalkulace-row');
        const newRow = template.cloneNode(true);

        newRow.querySelectorAll('.custom-select-wrapper').forEach(wrapper => {
            const nativeSelect = wrapper.querySelector('select');
            if (nativeSelect) {
                wrapper.parentNode.insertBefore(nativeSelect, wrapper);
            }
            wrapper.remove();
        });

        newRow.querySelectorAll('select').forEach(s => {
            s.selectedIndex = 0;
            s.style.display = 'block';
        });

        container.appendChild(newRow);

        initializeRow(newRow);
    };
};

function initializeRow(row) {
    row.querySelectorAll('select').forEach(select => {
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

function createCustomDropdown(select) {
    select.style.display = 'none';

    const wrapper = document.createElement('div');
    wrapper.className = 'custom-select-wrapper';
    select.parentNode.insertBefore(wrapper, select);
    wrapper.appendChild(select);

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

document.addEventListener('click', () => {
    document.querySelectorAll('.custom-select-menu').forEach(m => m.classList.remove('open'));
    document.querySelectorAll('.custom-select-display').forEach(d => d.classList.remove('open'));
});

document.addEventListener("DOMContentLoaded", initFormLogic);
document.addEventListener("turbo:load", initFormLogic);
document.addEventListener("turbolinks:load", initFormLogic);
