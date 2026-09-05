CREATE SCHEMA catalog;

CREATE TYPE catalog.account_state AS ENUM ('active', 'suspended');

CREATE TABLE catalog.accounts (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    email text NOT NULL,
    state catalog.account_state NOT NULL DEFAULT 'active'
);

COMMENT ON TABLE catalog.accounts IS 'Application accounts';

CREATE VIEW catalog.active_accounts AS
SELECT id, email
FROM catalog.accounts
WHERE state = 'active';
