CREATE TABLE shared_name (
    id INTEGER PRIMARY KEY,
    metric REAL NOT NULL
);

CREATE VIEW metric_names AS
SELECT id, metric FROM shared_name;
