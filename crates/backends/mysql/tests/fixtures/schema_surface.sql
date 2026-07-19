CREATE TABLE tenants (
    tenant_id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    name VARCHAR(120) NOT NULL COMMENT 'Tenant display name',
    PRIMARY KEY (tenant_id),
    UNIQUE KEY tenants_name_uq (name)
) ENGINE=InnoDB COMMENT='Application tenants';

CREATE TABLE accounts (
    tenant_id BIGINT UNSIGNED NOT NULL,
    account_id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    email VARCHAR(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL,
    normalized_email VARCHAR(255) GENERATED ALWAYS AS (lower(email)) STORED,
    secret_token VARCHAR(64) INVISIBLE,
    status ENUM('active','disabled') NOT NULL DEFAULT 'active',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (account_id),
    CONSTRAINT accounts_tenant_fk FOREIGN KEY (tenant_id) REFERENCES tenants (tenant_id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT accounts_email_check CHECK (email <> ''),
    UNIQUE KEY accounts_tenant_email_uq (tenant_id, email(120)),
    KEY accounts_email_desc_idx (email DESC) INVISIBLE,
    KEY accounts_normalized_idx ((lower(email)))
) ENGINE=InnoDB ROW_FORMAT=DYNAMIC COMMENT='User accounts';

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
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

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
DO DELETE FROM accounts WHERE status = 'disabled' AND updated_at < CURRENT_TIMESTAMP - INTERVAL 365 DAY;
