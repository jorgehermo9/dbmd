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
    embedding VECTOR(3),
    default_embedding VECTOR,
    home POINT NOT NULL SRID 4326,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (account_id),
    CONSTRAINT accounts_tenant_fk FOREIGN KEY (tenant_id) REFERENCES tenants (tenant_id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT accounts_email_check CHECK (email <> ''),
    CONSTRAINT accounts_status_check CHECK (status IN ('active', 'disabled')) NOT ENFORCED,
    UNIQUE KEY accounts_tenant_email_uq (tenant_id, email(120)),
    KEY accounts_email_desc_idx (email DESC) COMMENT 'Descending email lookup' INVISIBLE,
    KEY accounts_normalized_idx ((lower(email))),
    FULLTEXT KEY accounts_email_ft (email),
    SPATIAL KEY accounts_home_spatial (home)
) ENGINE=InnoDB ROW_FORMAT=DYNAMIC COMMENT='User accounts';

CREATE TABLE inline_memberships (
    membership_id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    tenant_id BIGINT UNSIGNED NOT NULL REFERENCES tenants ON DELETE CASCADE
) ENGINE=InnoDB COMMENT='Exercises MySQL 9 inline implicit-parent foreign keys';

CREATE TABLE monthly_metrics (
    occurred_on DATE NOT NULL,
    metric VARCHAR(64) NOT NULL,
    value BIGINT NOT NULL,
    PRIMARY KEY (occurred_on, metric)
) ENGINE=InnoDB
PARTITION BY RANGE (YEAR(occurred_on))
SUBPARTITION BY HASH (MONTH(occurred_on))
SUBPARTITIONS 2 (
    PARTITION p2025 VALUES LESS THAN (2026)
        (SUBPARTITION p2025_h0, SUBPARTITION p2025_h1),
    PARTITION pmax VALUES LESS THAN MAXVALUE
        (SUBPARTITION pmax_h0, SUBPARTITION pmax_h1)
);

CREATE TABLE memory_lookup (
    lookup_key VARCHAR(64) NOT NULL,
    payload VARCHAR(255),
    PRIMARY KEY (lookup_key),
    KEY memory_payload_hash (payload) USING HASH
) ENGINE=MEMORY;

SET SESSION sql_generate_invisible_primary_key = ON;
CREATE TABLE generated_primary_key (
    payload VARCHAR(64)
) ENGINE=InnoDB;
SET SESSION sql_generate_invisible_primary_key = OFF;

CREATE SQL SECURITY INVOKER VIEW active_accounts
AS SELECT tenant_id, account_id, email FROM accounts WHERE status = 'active'
WITH CASCADED CHECK OPTION;

CREATE JSON RELATIONAL DUALITY VIEW tenant_documents
AS SELECT JSON_DUALITY_OBJECT('_id':tenant_id, 'name':name)
FROM tenants;

CREATE TRIGGER accounts_updated
BEFORE UPDATE ON accounts
FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER accounts_update_marker
BEFORE UPDATE ON accounts
FOR EACH ROW FOLLOWS accounts_updated
SET @dbmd_last_account = NEW.account_id;

CREATE FUNCTION normalize_email(value VARCHAR(255))
RETURNS VARCHAR(255) DETERMINISTIC NO SQL
RETURN lower(value);

CREATE PROCEDURE disable_account(IN target_id BIGINT UNSIGNED)
MODIFIES SQL DATA
UPDATE accounts SET status = 'disabled' WHERE account_id = target_id;

CREATE PROCEDURE next_account_id(IN current_id BIGINT UNSIGNED, OUT next_id BIGINT UNSIGNED)
NO SQL
SET next_id = current_id + 1;

CREATE EVENT purge_disabled_accounts
ON SCHEDULE EVERY 1 DAY
STARTS '2030-01-01 00:00:00'
ON COMPLETION PRESERVE
DISABLE
COMMENT 'Remove old disabled accounts'
DO DELETE FROM accounts WHERE status = 'disabled' AND updated_at < CURRENT_TIMESTAMP - INTERVAL 365 DAY;

CREATE EVENT archive_accounts_once
ON SCHEDULE AT '2031-01-01 00:00:00'
ON COMPLETION PRESERVE
DISABLE
DO SET @dbmd_archive_requested = 1;
