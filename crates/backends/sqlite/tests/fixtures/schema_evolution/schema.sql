CREATE TABLE draft_records (
    id INTEGER PRIMARY KEY,
    old_name TEXT,
    obsolete TEXT
);

CREATE INDEX draft_records_obsolete_idx ON draft_records (obsolete);
CREATE VIEW obsolete_view AS SELECT obsolete FROM draft_records;
CREATE TRIGGER obsolete_trigger AFTER INSERT ON draft_records BEGIN SELECT 1; END;

DROP INDEX draft_records_obsolete_idx;
DROP VIEW obsolete_view;
DROP TRIGGER obsolete_trigger;

ALTER TABLE draft_records RENAME TO records;
ALTER TABLE records RENAME COLUMN old_name TO name;
ALTER TABLE records ADD COLUMN normalized_name TEXT GENERATED ALWAYS AS (lower(name)) VIRTUAL;
ALTER TABLE records DROP COLUMN obsolete;
ALTER TABLE records ALTER name SET NOT NULL;
ALTER TABLE records ADD COLUMN optional_note TEXT;
ALTER TABLE records ALTER optional_note SET NOT NULL;
ALTER TABLE records ALTER optional_note DROP NOT NULL;

CREATE TABLE imported_records AS
SELECT 1 AS id, 'first' AS label;

CREATE TABLE discarded (id INTEGER);
DROP TABLE discarded;

CREATE TEMP TABLE connection_only (id INTEGER PRIMARY KEY);
CREATE TEMP VIEW connection_only_view AS SELECT id FROM connection_only;
CREATE TEMP TRIGGER connection_only_trigger AFTER INSERT ON connection_only BEGIN SELECT 1; END;
CREATE TEMP TRIGGER connection_only_main_trigger AFTER UPDATE ON main.records BEGIN SELECT 1; END;
