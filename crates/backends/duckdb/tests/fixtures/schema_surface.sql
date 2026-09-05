CREATE SCHEMA analytics;

CREATE TYPE analytics.account_status AS ENUM ('active', 'disabled');
COMMENT ON TYPE analytics.account_status IS 'Account lifecycle';
CREATE TYPE analytics.account_pair AS STRUCT(account_id BIGINT, tenant_id BIGINT);
CREATE TYPE analytics.reference_value AS UNION(account_id BIGINT, external_id VARCHAR);
CREATE TYPE analytics.positive_integer AS INTEGER;

CREATE SEQUENCE analytics.account_id_seq START 1000 INCREMENT 10;
COMMENT ON SEQUENCE analytics.account_id_seq IS 'Account identifiers';

CREATE TABLE analytics.tenants (
    tenant_id BIGINT PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE
);
COMMENT ON TABLE analytics.tenants IS 'Application tenants';

CREATE TABLE analytics.accounts (
    tenant_id BIGINT NOT NULL REFERENCES analytics.tenants (tenant_id),
    account_id BIGINT PRIMARY KEY DEFAULT nextval('analytics.account_id_seq'),
    email VARCHAR NOT NULL,
    normalized_email VARCHAR GENERATED ALWAYS AS (lower(email)) VIRTUAL,
    status analytics.account_status NOT NULL DEFAULT 'active',
    balance DECIMAL(18, 2) NOT NULL DEFAULT 0,
    metadata STRUCT(source VARCHAR, tags VARCHAR[]),
    typed_pair analytics.account_pair,
    typed_reference analytics.reference_value,
    retry_count analytics.positive_integer,
    CONSTRAINT accounts_email_uq UNIQUE (tenant_id, email),
    CONSTRAINT accounts_balance_check CHECK (balance >= 0)
);
COMMENT ON TABLE analytics.accounts IS 'User accounts';
COMMENT ON COLUMN analytics.accounts.email IS 'Canonical email address';

CREATE INDEX accounts_email_idx ON analytics.accounts (email);

CREATE VIEW analytics.active_accounts AS
SELECT tenant_id, account_id, email
FROM analytics.accounts
WHERE status = 'active';
COMMENT ON VIEW analytics.active_accounts IS 'Active accounts only';

CREATE MACRO analytics.normalize_email(value) AS lower(value);
CREATE MACRO analytics.accounts_for_tenant(owner_id) AS TABLE
SELECT * FROM analytics.accounts WHERE tenant_id = owner_id;
