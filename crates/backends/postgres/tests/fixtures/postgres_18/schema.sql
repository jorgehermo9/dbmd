CREATE SCHEMA temporal;
CREATE EXTENSION btree_gist;
COMMENT ON EXTENSION btree_gist IS 'Temporal exclusion operator support';
CREATE COLLATION temporal.unicode_fast (
    provider = builtin,
    locale = 'PG_UNICODE_FAST'
);
COMMENT ON COLLATION temporal.unicode_fast IS 'Fast Unicode semantics';

CREATE TABLE temporal.accounts (
    account_id bigint,
    email text COLLATE temporal.unicode_fast CONSTRAINT accounts_email_required NOT NULL,
    base_amount integer,
    virtual_amount integer GENERATED ALWAYS AS (base_amount * 2) VIRTUAL,
    stored_amount integer GENERATED ALWAYS AS (base_amount * 3) STORED,
    CONSTRAINT accounts_amount_nonnegative
        CHECK (base_amount >= 0) NOT ENFORCED
);

CREATE TABLE temporal.plan_versions (
    plan_id bigint NOT NULL,
    valid_at daterange NOT NULL,
    CONSTRAINT plan_versions_identity
        UNIQUE (plan_id, valid_at WITHOUT OVERLAPS)
);

CREATE TABLE temporal.plan_assignments (
    plan_id bigint NOT NULL,
    valid_at daterange NOT NULL,
    CONSTRAINT assignments_plan_period
        FOREIGN KEY (plan_id, PERIOD valid_at)
        REFERENCES temporal.plan_versions (plan_id, PERIOD valid_at)
        NOT ENFORCED
);

CREATE PUBLICATION temporal_changes
    FOR TABLE temporal.accounts (account_id, stored_amount) WHERE (base_amount >= 0)
    WITH (publish = 'insert', publish_generated_columns = 'stored');
COMMENT ON PUBLICATION temporal_changes IS 'Stored generated values for analytics';

CREATE PUBLICATION temporal_schema
    FOR TABLES IN SCHEMA temporal
    WITH (publish = 'insert, truncate');

CREATE PUBLICATION all_tables
    FOR ALL TABLES
    WITH (publish = 'insert');
