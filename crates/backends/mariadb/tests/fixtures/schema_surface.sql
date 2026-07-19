CREATE SEQUENCE order_number_seq START WITH 1000 INCREMENT BY 10 CACHE 20;

CREATE TABLE tenants (
    tenant_id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    name VARCHAR(120) NOT NULL COMMENT 'Tenant display name',
    PRIMARY KEY (tenant_id),
    UNIQUE KEY tenants_name_uq (name)
) ENGINE=InnoDB COMMENT='Application tenants';

CREATE TABLE accounts (
    tenant_id BIGINT UNSIGNED NOT NULL,
    account_id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    email VARCHAR(255) NOT NULL,
    normalized_email VARCHAR(255) AS (lower(email)) PERSISTENT,
    status ENUM('active','disabled') NOT NULL DEFAULT 'active',
    row_start TIMESTAMP(6) GENERATED ALWAYS AS ROW START,
    row_end TIMESTAMP(6) GENERATED ALWAYS AS ROW END,
    PERIOD FOR SYSTEM_TIME (row_start, row_end),
    PRIMARY KEY (account_id),
    CONSTRAINT accounts_tenant_fk FOREIGN KEY (tenant_id) REFERENCES tenants (tenant_id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT accounts_email_check CHECK (email <> ''),
    UNIQUE KEY accounts_tenant_email_uq (tenant_id, email(120)),
    KEY accounts_email_ignored_idx (email) IGNORED
) ENGINE=InnoDB WITH SYSTEM VERSIONING COMMENT='Versioned user accounts';

CREATE TABLE monthly_metrics (
    occurred_on DATE NOT NULL,
    metric VARCHAR(64) NOT NULL,
    value BIGINT NOT NULL,
    PRIMARY KEY (occurred_on, metric)
) ENGINE=InnoDB
PARTITION BY RANGE (YEAR(occurred_on)) (
    PARTITION p2025 VALUES LESS THAN (2026),
    PARTITION pmax VALUES LESS THAN MAXVALUE
);

CREATE SQL SECURITY INVOKER VIEW active_accounts
AS SELECT tenant_id, account_id, email FROM accounts WHERE status = 'active'
WITH CASCADED CHECK OPTION;

CREATE TRIGGER accounts_updated
BEFORE UPDATE ON accounts
FOR EACH ROW SET NEW.status = COALESCE(NEW.status, OLD.status);

CREATE FUNCTION normalize_email(value VARCHAR(255))
RETURNS VARCHAR(255) DETERMINISTIC NO SQL
RETURN lower(value);

CREATE PROCEDURE disable_account(IN target_id BIGINT UNSIGNED)
MODIFIES SQL DATA
UPDATE accounts SET status = 'disabled' WHERE account_id = target_id;

CREATE EVENT purge_disabled_accounts
ON SCHEDULE EVERY 1 DAY
STARTS '2030-01-01 00:00:00'
ON COMPLETION PRESERVE
DISABLE
COMMENT 'Remove old disabled accounts'
DO DELETE FROM accounts WHERE status = 'disabled' AND row_end < CURRENT_TIMESTAMP - INTERVAL 365 DAY;
