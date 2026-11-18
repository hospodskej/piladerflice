-- Which e-shop products a firewood section of the price list ("Jasan / Dub")
-- stands for. The section's availability is read from those products'
-- variants: loose firewood from the container variants, crated firewood
-- from the crate variants. Set in Admin › Ceník.
CREATE TABLE pricelist_stock_links (
    category           TEXT    NOT NULL CHECK (category IN ('palivove_volne', 'palivove_skladane')),
    subcategory        TEXT    NOT NULL,
    catalog_product_id INTEGER NOT NULL REFERENCES catalog_products (id) ON DELETE CASCADE,
    PRIMARY KEY (category, subcategory, catalog_product_id)
);

-- Link the sections already in the price list to the woods they name
-- ("Ostatní tvrdé", other hardwoods, is the hornbeam).
INSERT OR IGNORE INTO pricelist_stock_links (category, subcategory, catalog_product_id)
SELECT DISTINCT i.category, i.subcategory, p.id
FROM pricelist_items i
JOIN catalog_products p ON p.category = 'palivove'
    AND (instr(i.subcategory, p.title) > 0
         OR (p.key = 'habr' AND instr(i.subcategory, 'Ostatní tvrdé') > 0))
WHERE i.category IN ('palivove_volne', 'palivove_skladane')
  AND i.subcategory IS NOT NULL AND i.subcategory <> '';
