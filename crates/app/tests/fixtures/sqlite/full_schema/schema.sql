CREATE TABLE organizations (
    tenant_id INTEGER NOT NULL,
    slug TEXT COLLATE NOCASE NOT NULL,
    CONSTRAINT organizations_pk PRIMARY KEY (tenant_id, slug)
) WITHOUT ROWID, STRICT;

CREATE TABLE accounts (
    id INTEGER CONSTRAINT accounts_pk PRIMARY KEY AUTOINCREMENT,
    tenant_id INTEGER NOT NULL,
    organization_slug TEXT NOT NULL,
    email TEXT COLLATE NOCASE CONSTRAINT accounts_email_key UNIQUE ON CONFLICT IGNORE,
    balance_cents INTEGER NOT NULL DEFAULT (0)
        CONSTRAINT accounts_balance_check CHECK (balance_cents >= 0),
    normalized_email TEXT GENERATED ALWAYS AS (lower(email)) STORED,
    CONSTRAINT accounts_organization_fk
        FOREIGN KEY (tenant_id, organization_slug)
        REFERENCES organizations
        MATCH simple
        ON UPDATE CASCADE
        ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT accounts_email_check CHECK (length(email) > 3)
) STRICT;

CREATE UNIQUE INDEX accounts_active_email_idx
    ON accounts (tenant_id, lower(email) COLLATE NOCASE DESC)
    WHERE balance_cents >= 0;

CREATE VIEW account_directory (account_id, email) AS
SELECT id, email FROM accounts;

CREATE VIEW account_balances AS
SELECT id, balance_cents FROM accounts;

CREATE TRIGGER account_directory_insert
INSTEAD OF INSERT ON account_directory
BEGIN
    INSERT INTO accounts (id, tenant_id, organization_slug, email)
    VALUES (NEW.account_id, 1, 'default', NEW.email);
END;

CREATE TRIGGER accounts_normalize_email
AFTER UPDATE OF email ON accounts
WHEN NEW.email != lower(NEW.email)
BEGIN
    UPDATE accounts SET email = lower(NEW.email) WHERE id = NEW.id;
END;

CREATE TRIGGER accounts_prevent_root_delete
BEFORE DELETE ON accounts
WHEN OLD.id = 0
BEGIN
    SELECT RAISE(IGNORE);
END;

CREATE VIRTUAL TABLE account_search USING fts5(email, content='accounts', content_rowid='id');

CREATE TABLE draft_migration_target (id INTEGER PRIMARY KEY, old_name TEXT, obsolete TEXT);
ALTER TABLE draft_migration_target RENAME TO migration_target;
ALTER TABLE migration_target RENAME COLUMN old_name TO name;
ALTER TABLE migration_target ADD COLUMN generated_name TEXT AS (upper(name)) VIRTUAL;
ALTER TABLE migration_target DROP COLUMN obsolete;
ALTER TABLE migration_target ALTER name SET NOT NULL;
ALTER TABLE migration_target ADD COLUMN optional_note TEXT;
ALTER TABLE migration_target ALTER optional_note SET NOT NULL;
ALTER TABLE migration_target ALTER optional_note DROP NOT NULL;

CREATE TABLE imported AS SELECT 1 AS id, 'seed' AS label;

CREATE TABLE removed_table (id INTEGER);
CREATE INDEX removed_index ON removed_table (id);
CREATE VIEW removed_view AS SELECT id FROM removed_table;
CREATE TRIGGER removed_trigger AFTER INSERT ON removed_table BEGIN SELECT 1; END;
DROP TRIGGER removed_trigger;
DROP VIEW removed_view;
DROP INDEX removed_index;
DROP TABLE removed_table;

CREATE TEMP TABLE connection_only (id INTEGER PRIMARY KEY);
CREATE TEMP VIEW connection_only_view AS SELECT id FROM connection_only;
CREATE TEMP TRIGGER connection_only_trigger AFTER INSERT ON connection_only BEGIN SELECT 1; END;
