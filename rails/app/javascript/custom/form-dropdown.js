// Shows an option's text with its number in blue ("5" + " PRM"), as
// elements with textContent: the text comes from the admin or from what the
// customer typed, and must never be read as HTML.
function showOptionText(element, text) {
    const parts = text.trim().split(' ');
    const number = document.createElement('span');
    number.className = 'custom-select-number';
    number.textContent = parts.shift();
    element.replaceChildren(number);
    if (parts.length) {
        const rest = document.createElement('span');
        rest.textContent = parts.join(' ');
        element.append(' ', rest);
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

document.addEventListener("DOMContentLoaded", () => {
    const selects = document.querySelectorAll(".kalkulace-grid-4 select, .kalkulace-grid-5 select");

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


        Array.from(originalSelect.options).forEach((opt, index) => {
            const item = document.createElement('div');
            item.className = 'custom-select-item';
            showOptionText(item, opt.text);

            item.addEventListener('click', (e) => {
                e.stopPropagation();
                if (opt.value === '__custom__' && originalSelect.value !== '__custom__') {
                    originalSelect.dataset.restoreIndex = originalSelect.selectedIndex;
                }
                originalSelect.selectedIndex = index;
                showOptionText(display, opt.text);
                menu.classList.remove('open');
                display.classList.remove('open');
                handleCustomOption(originalSelect, opt.value);
            });
            menu.appendChild(item);
        });

        showOptionText(display, originalSelect.options[originalSelect.selectedIndex].text);

        const customInput = originalSelect.closest('.form-group')?.querySelector('.custom-value-input');
        if (customInput) {
            const presetOptions = Array.from(originalSelect.options).filter(o => o.value !== '__custom__');
            const isNumericField = presetOptions.length > 0 && presetOptions.every(o => extractUnit(o.text) !== '');
            if (isNumericField) {
                originalSelect.dataset.numericField = 'true';
                customInput.inputMode = 'decimal';
                customInput.addEventListener('input', () => {
                    const sanitized = customInput.value.replace(/[^0-9.,]/g, '');
                    if (sanitized !== customInput.value) customInput.value = sanitized;
                });
            }

            customInput.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') {
                    e.preventDefault();
                    customInput.blur();
                }
            });

            customInput.addEventListener('blur', () => {
                let value = customInput.value.trim();

                if (!value) {
                    const restoreIndex = parseInt(originalSelect.dataset.restoreIndex ?? '0', 10);
                    originalSelect.selectedIndex = restoreIndex;
                    showOptionText(display, originalSelect.options[restoreIndex].text);
                    handleCustomOption(originalSelect, originalSelect.options[restoreIndex].value);
                    return;
                }

                if (/^\d+(?:[.,]\d+)?$/.test(value)) {
                    const referenceIndex = parseInt(originalSelect.dataset.restoreIndex ?? '0', 10);
                    const unit = extractUnit(originalSelect.options[referenceIndex].text);
                    if (unit) value = `${value} ${unit}`;
                }

                customInput.value = value;
                customInput.style.display = 'none';
                showOptionText(display, value);
            });
        }

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
