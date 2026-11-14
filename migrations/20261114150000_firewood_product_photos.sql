-- The firewood products were seeded with the photos of the "Sortiment" page;
-- the e-shop has its own photo of each wood. Only rows still holding the
-- seeded path change, so a photo uploaded in the admin is kept.
UPDATE catalog_products
SET image = '/img/eshop/' || substr(image, length('/img/sortiment/palivove-drevo/') + 1)
WHERE image IN (
    '/img/sortiment/palivove-drevo/smrk.webp',
    '/img/sortiment/palivove-drevo/dub.webp',
    '/img/sortiment/palivove-drevo/buk.webp',
    '/img/sortiment/palivove-drevo/akat.webp',
    '/img/sortiment/palivove-drevo/jasan.webp',
    '/img/sortiment/palivove-drevo/habr.webp'
);
