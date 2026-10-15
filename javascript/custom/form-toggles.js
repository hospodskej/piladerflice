const initFormLogic = () => {
    const toggleBtns = document.querySelectorAll(".toggle-btn");
    const categoryInput = document.getElementById('kalkulace-category-input');

    toggleBtns.forEach(btn => {
        btn.onclick = function() {
            toggleBtns.forEach(b => b.classList.remove("active"));
            this.classList.add("active");

            const target = this.dataset.target;
            if (categoryInput) categoryInput.value = target;

            document.querySelectorAll('.kalkulace-fields-section').forEach(section => {
                const isTarget = section.dataset.category === target;
                section.hidden = !isTarget;
                section.disabled = !isTarget;
            });
        };
    });

    document.querySelectorAll('.kalkulace-fields-section').forEach(section => initFieldsSection(section));
};

function initFieldsSection(section) {
    const container = section.querySelector('.kalkulace-items-container');
    const addBtn = section.querySelector('.add-kalkulace-row');

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
            s.disabled = false;
        });

        newRow.querySelectorAll('.custom-value-input').forEach(input => {
            input.value = '';
            input.name = '';
            input.required = false;
            input.style.display = 'none';
        });

        container.appendChild(newRow);

        initializeRow(newRow);
    };
}

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

function extractUnit(text) {
    const match = text.trim().match(/^\d+(?:[.,]\d+)?\s*(.*)$/);
    return match ? match[1].trim() : '';
}

function handleCustomOption(select, optionValue) {
    const customInput = select.closest('.form-group')?.querySelector('.custom-value-input');
    if (!customInput) return;

    if (optionValue === '__custom__') {
        if (select.dataset.numericField === 'true') {
            const match = customInput.value.match(/^\s*(\d+(?:[.,]\d+)?)/);
            customInput.value = match ? match[1] : '';
        }
        customInput.style.display = 'block';
        customInput.name = select.name;
        customInput.required = true;
        select.disabled = true;
        customInput.focus();
    } else {
        customInput.style.display = 'none';
        customInput.name = '';
        customInput.required = false;
        select.disabled = false;
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
            if (opt.value === '__custom__' && select.value !== '__custom__') {
                select.dataset.restoreIndex = select.selectedIndex;
            }
            select.selectedIndex = index;
            display.innerHTML = formatText(opt.text);
            menu.classList.remove('open');
            display.classList.remove('open');
            handleCustomOption(select, opt.value);
        };
        menu.appendChild(item);
    });

    display.innerHTML = formatText(select.options[select.selectedIndex].text);

    const customInput = select.closest('.form-group')?.querySelector('.custom-value-input');
    if (customInput) {
        const presetOptions = Array.from(select.options).filter(o => o.value !== '__custom__');
        const isNumericField = presetOptions.length > 0 && presetOptions.every(o => extractUnit(o.text) !== '');
        if (isNumericField) {
            select.dataset.numericField = 'true';
            customInput.inputMode = 'decimal';
            customInput.oninput = () => {
                const sanitized = customInput.value.replace(/[^0-9.,]/g, '');
                if (sanitized !== customInput.value) customInput.value = sanitized;
            };
        }

        customInput.onkeydown = (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                customInput.blur();
            }
        };

        customInput.onblur = () => {
            let value = customInput.value.trim();

            if (!value) {
                const restoreIndex = parseInt(select.dataset.restoreIndex ?? '0', 10);
                select.selectedIndex = restoreIndex;
                display.innerHTML = formatText(select.options[restoreIndex].text);
                handleCustomOption(select, select.options[restoreIndex].value);
                return;
            }

            if (/^\d+(?:[.,]\d+)?$/.test(value)) {
                const referenceIndex = parseInt(select.dataset.restoreIndex ?? '0', 10);
                const unit = extractUnit(select.options[referenceIndex].text);
                if (unit) value = `${value} ${unit}`;
            }

            customInput.value = value;
            customInput.style.display = 'none';
            display.innerHTML = formatText(value);
        };
    }

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
