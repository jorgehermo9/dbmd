PRAGMA foreign_keys = ON;

CREATE TABLE organizations (
    organization_id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
) STRICT;

CREATE TABLE accounts (
    account_id INTEGER PRIMARY KEY,
    organization_id INTEGER NOT NULL,
    email TEXT NOT NULL COLLATE NOCASE,
    normalized_email TEXT GENERATED ALWAYS AS (lower(email)) STORED,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    FOREIGN KEY (organization_id) REFERENCES organizations (organization_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    UNIQUE (organization_id, email)
) STRICT;

CREATE INDEX accounts_active_email_idx
    ON accounts (normalized_email)
    WHERE status = 'active';

CREATE VIEW active_accounts AS
SELECT account_id, organization_id, email
FROM accounts
WHERE status = 'active';
