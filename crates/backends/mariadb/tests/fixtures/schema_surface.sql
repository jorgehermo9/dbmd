ALTER DATABASE test COMMENT = 'Commerce schema fixture';

INSTALL SONAME 'ha_blackhole';

CREATE USER 'analytics_service'@'localhost'
IDENTIFIED WITH caching_sha2_password
USING '$A$005$abcdefghijklmnopqrstABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopq'
REQUIRE SUBJECT '/CN=dbmd-client' AND ISSUER '/CN=dbmd-ca' AND CIPHER 'TLS_AES_256_GCM_SHA384'
WITH MAX_QUERIES_PER_HOUR 17 MAX_USER_CONNECTIONS 3;
ALTER USER 'analytics_service'@'localhost'
PASSWORD EXPIRE INTERVAL 90 DAY ACCOUNT LOCK;
CREATE ROLE analytics_reader;
CREATE USER 'proxy_target'@'localhost' ACCOUNT LOCK;
GRANT SELECT ON test.* TO analytics_reader;
GRANT analytics_reader TO 'analytics_service'@'localhost' WITH ADMIN OPTION;
SET DEFAULT ROLE analytics_reader FOR 'analytics_service'@'localhost';
GRANT PROXY ON 'proxy_target'@'localhost' TO 'analytics_service'@'localhost' WITH GRANT OPTION;

CREATE SERVER analytics_remote
FOREIGN DATA WRAPPER mariadb
OPTIONS (
    HOST 'db.internal',
    DATABASE 'analytics',
    USER 'reader',
    PASSWORD 'dbmd-mariadb-server-secret-sentinel',
    PORT 3307,
    OWNER 'platform',
    REGION 'eu-west-1'
);

CREATE SEQUENCE order_number_seq START WITH 1000 INCREMENT BY 10 CACHE 20;
CREATE SEQUENCE descending_order_seq START WITH 0 INCREMENT BY -2 MINVALUE -20 MAXVALUE 0 NOCACHE CYCLE;

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
    profile_document XMLTYPE COMMENT 'MariaDB 12.3 XML profile payload',
    status ENUM('active','disabled') NOT NULL DEFAULT 'active',
    secret_token VARCHAR(64) INVISIBLE,
    home POINT NOT NULL,
    row_start TIMESTAMP(6) GENERATED ALWAYS AS ROW START,
    row_end TIMESTAMP(6) GENERATED ALWAYS AS ROW END,
    PERIOD FOR SYSTEM_TIME (row_start, row_end),
    PRIMARY KEY (account_id),
    CONSTRAINT accounts_tenant_fk FOREIGN KEY (tenant_id) REFERENCES tenants (tenant_id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT accounts_email_check CHECK (email <> ''),
    UNIQUE KEY accounts_tenant_email_uq (tenant_id, email(120)),
    KEY accounts_status_desc_idx (status DESC) COMMENT 'Status lookup ordering',
    KEY accounts_email_ignored_idx (email) IGNORED,
    FULLTEXT KEY accounts_email_fulltext (email),
    SPATIAL KEY accounts_home_spatial (home)
) ENGINE=InnoDB WITH SYSTEM VERSIONING COMMENT='Versioned user accounts';

CREATE TABLE tenant_audits (
    audit_id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    tenant_id BIGINT UNSIGNED NOT NULL,
    CONSTRAINT accounts_tenant_fk FOREIGN KEY (tenant_id)
        REFERENCES tenants (tenant_id) ON DELETE CASCADE
) ENGINE=InnoDB COMMENT='Reuses a foreign-key name in the same schema';

CREATE TABLE tenant_embeddings (
    tenant_id BIGINT UNSIGNED NOT NULL,
    valid_from DATE NOT NULL,
    valid_to DATE NOT NULL,
    embedding VECTOR(5) NOT NULL,
    row_start TIMESTAMP(6) GENERATED ALWAYS AS ROW START,
    row_end TIMESTAMP(6) GENERATED ALWAYS AS ROW END,
    PERIOD FOR validity (valid_from, valid_to),
    PERIOD FOR SYSTEM_TIME (row_start, row_end),
    UNIQUE KEY tenant_validity_uq (tenant_id, validity WITHOUT OVERLAPS),
    VECTOR INDEX embedding_vector_idx (embedding) M=8 DISTANCE=cosine
) ENGINE=InnoDB WITH SYSTEM VERSIONING COMMENT='Bitemporal tenant embeddings';

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

CREATE TABLE discarded_events (
    event_id BIGINT NOT NULL
) ENGINE=BLACKHOLE COMMENT='Exercises an installed storage-engine plugin';

CREATE ALGORITHM=MERGE SQL SECURITY INVOKER VIEW active_accounts
AS SELECT tenant_id, account_id, email FROM accounts WHERE status = 'active'
WITH CASCADED CHECK OPTION;

CREATE TRIGGER accounts_updated
BEFORE UPDATE OF status, email ON accounts
FOR EACH ROW SET NEW.status = COALESCE(NEW.status, OLD.status);

CREATE TRIGGER accounts_update_marker
BEFORE UPDATE ON accounts
FOR EACH ROW FOLLOWS accounts_updated
SET @dbmd_mariadb_last_account = NEW.account_id;

CREATE TRIGGER accounts_changed
AFTER INSERT OR UPDATE OR DELETE ON accounts
FOR EACH ROW SET @dbmd_mariadb_account_changed = 1;

CREATE FUNCTION normalize_email(value VARCHAR(255) DEFAULT 'fallback@example.invalid')
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
DO DELETE FROM accounts WHERE status = 'disabled' AND row_end < CURRENT_TIMESTAMP - INTERVAL 365 DAY;

CREATE EVENT archive_accounts_once
ON SCHEDULE AT '2031-01-01 00:00:00'
ON COMPLETION PRESERVE
DISABLE
DO SET @dbmd_mariadb_archive_requested = 1;

DELIMITER //
CREATE PACKAGE analytics_tools
COMMENT 'Analytics package'
SQL SECURITY INVOKER
PROCEDURE refresh_cache(IN tenant BIGINT);
FUNCTION normalize(value VARCHAR(255)) RETURNS VARCHAR(255);
END//

CREATE PACKAGE BODY analytics_tools
PROCEDURE refresh_cache(IN tenant BIGINT)
BEGIN
    SELECT tenant;
END;
FUNCTION normalize(value VARCHAR(255)) RETURNS VARCHAR(255)
RETURN lower(value);
END//
DELIMITER ;

GRANT EXECUTE ON FUNCTION normalize_email TO analytics_reader;
GRANT SHOW CREATE ROUTINE ON test.* TO analytics_reader;
GRANT EXECUTE ON PACKAGE analytics_tools TO analytics_reader;
