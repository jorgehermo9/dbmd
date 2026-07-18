CREATE TABLE catalog_items (
    id INTEGER PRIMARY KEY,
    sku TEXT COLLATE NOCASE CONSTRAINT catalog_items_sku_key UNIQUE ON CONFLICT IGNORE,
    category TEXT NOT NULL,
    name TEXT NOT NULL,
    price_cents INTEGER NOT NULL
);

CREATE INDEX z_catalog_items_category_name
    ON catalog_items (category ASC, name COLLATE RTRIM DESC);

CREATE UNIQUE INDEX a_catalog_items_search_key
    ON catalog_items (category, lower(name) COLLATE NOCASE DESC)
    WHERE price_cents > 0;
