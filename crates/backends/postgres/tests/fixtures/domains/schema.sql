CREATE SCHEMA types;
CREATE ROLE domain_owner NOLOGIN;

CREATE DOMAIN types.email_address AS text
    COLLATE "C"
    DEFAULT ''
    NOT NULL
    CONSTRAINT email_shape CHECK (VALUE ~ '^[^@]+@[^@]+$');

ALTER DOMAIN types.email_address
    ADD CONSTRAINT email_not_blocked
    CHECK (VALUE <> 'blocked@example.com')
    NOT VALID;

ALTER DOMAIN types.email_address OWNER TO domain_owner;

COMMENT ON DOMAIN types.email_address IS
    'Canonical application email address';
COMMENT ON CONSTRAINT email_shape ON DOMAIN types.email_address IS
    'Requires one at-sign';

CREATE TABLE types.accounts (
    email types.email_address
);
