CREATE TABLE accounts (
    id INTEGER CONSTRAINT accounts_pk
        PRIMARY KEY ON CONFLICT REPLACE AUTOINCREMENT,
    email TEXT COLLATE NOCASE
        CONSTRAINT accounts_email_nn NOT NULL ON CONFLICT FAIL
        CONSTRAINT accounts_email_uq UNIQUE ON CONFLICT IGNORE,
    display_name TEXT CONSTRAINT accounts_display_name_uq UNIQUE ON CONFLICT ABORT,
    balance_cents INTEGER DEFAULT (0)
        CONSTRAINT accounts_balance_check CHECK (balance_cents >= 0),
    normalized_email TEXT
        GENERATED ALWAYS AS (lower(email)) STORED,
    parent_id INTEGER
        CONSTRAINT accounts_parent_fk REFERENCES accounts (id)
        MATCH simple
        ON UPDATE SET NULL
        ON DELETE CASCADE
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT accounts_email_check CHECK (length(email) > 3),
    CONSTRAINT accounts_email_balance_uq
        UNIQUE (email, balance_cents) ON CONFLICT ROLLBACK
);

CREATE TABLE parent_keys (
    tenant_id INTEGER NOT NULL,
    external_id TEXT NOT NULL,
    PRIMARY KEY (tenant_id, external_id)
) WITHOUT ROWID;

CREATE TABLE child_keys (
    tenant_id INTEGER,
    parent_external_id TEXT,
    FOREIGN KEY (tenant_id, parent_external_id)
        REFERENCES parent_keys
        ON DELETE SET DEFAULT
        NOT DEFERRABLE INITIALLY IMMEDIATE
);
