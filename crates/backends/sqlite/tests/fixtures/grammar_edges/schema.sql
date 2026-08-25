CREATE TABLE "parent keys" (
    id INTEGER PRIMARY KEY,
    "code a" TEXT CONSTRAINT parent_code_a_key UNIQUE,
    "code b" TEXT CONSTRAINT parent_code_b_key UNIQUE
);

CREATE TABLE "child refs" (
    id INTEGER PRIMARY KEY,
    "shared ref" TEXT DEFAULT 'missing',
    CONSTRAINT "fk code a"
        FOREIGN KEY ("shared ref") REFERENCES "parent keys" ("code a")
        MATCH FULL
        ON UPDATE NO ACTION
        ON DELETE CASCADE
        DEFERRABLE INITIALLY IMMEDIATE,
    CONSTRAINT "fk code b"
        FOREIGN KEY ("shared ref") REFERENCES "parent keys" ("code b")
        MATCH PARTIAL
        ON UPDATE RESTRICT
        ON DELETE SET DEFAULT
        NOT DEFERRABLE
);

CREATE TABLE sqliteXascending (
    id INTEGER PRIMARY KEY ASC,
    payload TEXT
);

CREATE TABLE sqliteXdescending (
    id INTEGER PRIMARY KEY DESC,
    payload TEXT
);

CREATE VIEW sqliteXview AS
SELECT id, payload FROM sqliteXascending;

CREATE TRIGGER sqliteXdefault_timing
UPDATE OF payload, intentionally_missing_column ON sqliteXascending
BEGIN
    SELECT NEW.payload;
END;
