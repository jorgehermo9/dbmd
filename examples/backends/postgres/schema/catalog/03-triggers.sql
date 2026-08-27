CREATE SCHEMA audit;

CREATE TABLE audit.accounts (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    email text NOT NULL,
    balance integer NOT NULL DEFAULT 0
);

CREATE TABLE audit.account_limits (
    account_id bigint PRIMARY KEY,
    minimum_balance integer NOT NULL
);

CREATE VIEW audit.account_emails AS
SELECT id, email
FROM audit.accounts;

COMMENT ON VIEW audit.account_emails IS 'Writable account email projection';

CREATE TABLE audit.partitioned_events (
    id bigint NOT NULL,
    occurred_on date NOT NULL
) PARTITION BY RANGE (occurred_on);

CREATE TABLE audit.partitioned_events_2026
PARTITION OF audit.partitioned_events
FOR VALUES FROM ('2026-01-01') TO ('2027-01-01');

CREATE FUNCTION audit.capture_row_change()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN COALESCE(NEW, OLD);
END;
$$;

CREATE FUNCTION audit.capture_statement_change()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN NULL;
END;
$$;

CREATE TRIGGER zz_accounts_change
BEFORE INSERT OR UPDATE OF email, balance OR DELETE ON audit.accounts
FOR EACH ROW
WHEN (pg_trigger_depth() = 0)
EXECUTE FUNCTION audit.capture_row_change('history', 'full');

COMMENT ON TRIGGER zz_accounts_change ON audit.accounts
IS 'Captures relevant account row changes';

ALTER TABLE audit.accounts ENABLE ALWAYS TRIGGER zz_accounts_change;

CREATE TRIGGER accounts_transition
AFTER UPDATE ON audit.accounts
REFERENCING OLD TABLE AS previous_rows NEW TABLE AS current_rows
FOR EACH STATEMENT
EXECUTE FUNCTION audit.capture_statement_change();

ALTER TABLE audit.accounts DISABLE TRIGGER accounts_transition;

CREATE CONSTRAINT TRIGGER accounts_balance_constraint
AFTER UPDATE OF balance ON audit.accounts
FROM audit.account_limits
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW
WHEN (NEW.balance < 0)
EXECUTE FUNCTION audit.capture_row_change('balance');

CREATE TRIGGER accounts_truncate
AFTER TRUNCATE ON audit.accounts
FOR EACH STATEMENT
EXECUTE FUNCTION audit.capture_statement_change();

ALTER TABLE audit.accounts ENABLE REPLICA TRIGGER accounts_truncate;

CREATE TRIGGER account_emails_write
INSTEAD OF INSERT OR UPDATE OR DELETE ON audit.account_emails
FOR EACH ROW
EXECUTE FUNCTION audit.capture_row_change('view');

CREATE TRIGGER partitioned_events_change
BEFORE INSERT OR UPDATE ON audit.partitioned_events
FOR EACH ROW
EXECUTE FUNCTION audit.capture_row_change('partition');
