CREATE SCHEMA zeta;
CREATE SCHEMA app;

CREATE TABLE zeta.audit_log (
    id bigint NOT NULL,
    recorded_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE app.accounts (
    id bigint GENERATED ALWAYS AS IDENTITY,
    email text NOT NULL DEFAULT 'unknown@example.invalid',
    normalized_email text GENERATED ALWAYS AS (lower(email)) STORED,
    CONSTRAINT accounts_email_nonempty CHECK (email <> '')
);

COMMENT ON TABLE app.accounts IS 'Application accounts';
COMMENT ON COLUMN app.accounts.email IS 'Canonical login address';

CREATE UNIQUE INDEX accounts_normalized_email_idx
    ON app.accounts USING btree (lower(email) DESC)
    WHERE email <> '';
