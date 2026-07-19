CREATE TABLE measurements (
    id INTEGER PRIMARY KEY NOT NULL,
    width REAL NOT NULL,
    height REAL NOT NULL,
    area REAL GENERATED ALWAYS AS (width * height) VIRTUAL,
    dimensions TEXT GENERATED ALWAYS AS (width || 'x' || height) STORED
) STRICT;

CREATE TABLE settings (
    key TEXT NOT NULL,
    locale TEXT NOT NULL,
    value TEXT,
    PRIMARY KEY (key, locale)
) WITHOUT ROWID;
