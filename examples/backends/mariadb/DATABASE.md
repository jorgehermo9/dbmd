# Database: `MariaDB commerce database`

## Schemas

| Name | Details |
|---|---|
| `test` | Default character set `utf8mb4`; collation `utf8mb4_uca1400_ai_ci`. Commerce schema fixture |


## Tables

### `test.accounts`

Versioned user accounts

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint(20) unsigned` | no | - |  |
| `account_id` | `bigint(20) unsigned` | no | - | `auto_increment` |
| `email` | `varchar(255)` | no | - |  |
| `normalized_email` | `varchar(255)` | yes | `NULL` | `STORED GENERATED`; stored generated as ``lcase(`email`)`` |
| `profile_document` | `xmltype` | yes | `NULL` | MariaDB 12.3 XML profile payload |
| `status` | `enum('active','disabled')` | no | `'active'` |  |
| `secret_token` | `varchar(64)` | yes | `NULL` | `INVISIBLE`; invisible |
| `home` | `point` | no | - |  |
| `row_start` | `timestamp(6)` | no | - | `STORED GENERATED`; stored generated as `ROW START`; system-time period start |
| `row_end` | `timestamp(6)` | no | - | `STORED GENERATED`; stored generated as `ROW END`; system-time period end |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `account_id, row_end` | - |
| `accounts_email_check` | `check` | `` | `` `email` <> '' ``; declared at table level |
| `accounts_tenant_email_uq` | `unique` | `tenant_id, email, row_end` | - |
| `accounts_tenant_fk` | `foreign_key` | `tenant_id` | references `test`.`tenants` (`tenant_id`); on update cascade; on delete restrict |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `account_id, row_end` | yes | BTREE | - |
| `accounts_email_fulltext` | `email` | no | FULLTEXT | - |
| `accounts_email_ignored_idx` | `email` | no | BTREE; ignored | - |
| `accounts_home_spatial` | `home(32)` | no | SPATIAL | - |
| `accounts_status_desc_idx` | `status DESC` | no | BTREE; comment `Status lookup ordering` | - |
| `accounts_tenant_email_uq` | `tenant_id, email(120), row_end` | yes | BTREE | - |


#### MariaDB

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_uca1400_ai_ci`

**System versioning:** enabled

**System-time period:** `row_start, row_end`

```sql
CREATE TABLE `accounts` (
  `tenant_id` bigint(20) unsigned NOT NULL,
  `account_id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `email` varchar(255) NOT NULL,
  `normalized_email` varchar(255) GENERATED ALWAYS AS (lcase(`email`)) STORED,
  `profile_document` xmltype DEFAULT NULL COMMENT 'MariaDB 12.3 XML profile payload',
  `status` enum('active','disabled') NOT NULL DEFAULT 'active',
  `secret_token` varchar(64) DEFAULT NULL INVISIBLE,
  `home` point NOT NULL,
  `row_start` timestamp(6) GENERATED ALWAYS AS ROW START,
  `row_end` timestamp(6) GENERATED ALWAYS AS ROW END,
  PRIMARY KEY (`account_id`,`row_end`),
  UNIQUE KEY `accounts_tenant_email_uq` (`tenant_id`,`email`(120),`row_end`),
  KEY `accounts_status_desc_idx` (`status` DESC) COMMENT 'Status lookup ordering',
  KEY `accounts_email_ignored_idx` (`email`) IGNORED,
  SPATIAL KEY `accounts_home_spatial` (`home`),
  FULLTEXT KEY `accounts_email_fulltext` (`email`),
  PERIOD FOR SYSTEM_TIME (`row_start`, `row_end`),
  CONSTRAINT `accounts_tenant_fk` FOREIGN KEY (`tenant_id`) REFERENCES `tenants` (`tenant_id`) ON UPDATE CASCADE,
  CONSTRAINT `accounts_email_check` CHECK (`email` <> '')
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci COMMENT='Versioned user accounts' WITH SYSTEM VERSIONING
```


### `test.discarded_events`

Exercises an installed storage-engine plugin

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `event_id` | `bigint(20)` | no | - |  |


#### MariaDB

**Engine:** `BLACKHOLE`

**Row format:** `Fixed`

**Collation:** `utf8mb4_uca1400_ai_ci`

```sql
CREATE TABLE `discarded_events` (
  `event_id` bigint(20) NOT NULL
) ENGINE=BLACKHOLE DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci COMMENT='Exercises an installed storage-engine plugin'
```


### `test.monthly_metrics`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `occurred_on` | `date` | no | - |  |
| `metric` | `varchar(64)` | no | - |  |
| `value` | `bigint(20)` | no | - |  |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `occurred_on, metric` | - |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `occurred_on, metric` | yes | BTREE | - |


#### MariaDB

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_uca1400_ai_ci`

**Create options:** `partitioned`

**Subpartition:** `p2025_h0`: method=`hash`, expression=``month(`occurred_on`)``, boundary=`2026`, position=1; nodegroup=`default`

**Subpartition:** `p2025_h1`: method=`hash`, expression=``month(`occurred_on`)``, boundary=`2026`, position=2; nodegroup=`default`

**Subpartition:** `pmax_h0`: method=`hash`, expression=``month(`occurred_on`)``, boundary=`MAXVALUE`, position=1; nodegroup=`default`

**Subpartition:** `pmax_h1`: method=`hash`, expression=``month(`occurred_on`)``, boundary=`MAXVALUE`, position=2; nodegroup=`default`

```sql
CREATE TABLE `monthly_metrics` (
  `occurred_on` date NOT NULL,
  `metric` varchar(64) NOT NULL,
  `value` bigint(20) NOT NULL,
  PRIMARY KEY (`occurred_on`,`metric`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci
 PARTITION BY RANGE (year(`occurred_on`))
SUBPARTITION BY HASH (month(`occurred_on`))
(PARTITION `p2025` VALUES LESS THAN (2026)
 (SUBPARTITION `p2025_h0` ENGINE = InnoDB,
  SUBPARTITION `p2025_h1` ENGINE = InnoDB),
 PARTITION `pmax` VALUES LESS THAN MAXVALUE
 (SUBPARTITION `pmax_h0` ENGINE = InnoDB,
  SUBPARTITION `pmax_h1` ENGINE = InnoDB))
```


### `test.tenants`

Application tenants

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint(20) unsigned` | no | - | `auto_increment` |
| `name` | `varchar(120)` | no | - | Tenant display name |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `tenant_id` | - |
| `tenants_name_uq` | `unique` | `name` | - |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `tenant_id` | yes | BTREE | - |
| `tenants_name_uq` | `name` | yes | BTREE | - |


#### MariaDB

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_uca1400_ai_ci`

```sql
CREATE TABLE `tenants` (
  `tenant_id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(120) NOT NULL COMMENT 'Tenant display name',
  PRIMARY KEY (`tenant_id`),
  UNIQUE KEY `tenants_name_uq` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci COMMENT='Application tenants'
```


### `test.tenant_audits`

Reuses a foreign-key name in the same schema

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `audit_id` | `bigint(20) unsigned` | no | - | `auto_increment` |
| `tenant_id` | `bigint(20) unsigned` | no | - |  |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `audit_id` | - |
| `accounts_tenant_fk` | `foreign_key` | `tenant_id` | references `test`.`tenants` (`tenant_id`); on update restrict; on delete cascade |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `audit_id` | yes | BTREE | - |
| `accounts_tenant_fk` | `tenant_id` | no | BTREE | - |


#### MariaDB

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_uca1400_ai_ci`

```sql
CREATE TABLE `tenant_audits` (
  `audit_id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint(20) unsigned NOT NULL,
  PRIMARY KEY (`audit_id`),
  KEY `accounts_tenant_fk` (`tenant_id`),
  CONSTRAINT `accounts_tenant_fk` FOREIGN KEY (`tenant_id`) REFERENCES `tenants` (`tenant_id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci COMMENT='Reuses a foreign-key name in the same schema'
```


### `test.tenant_embeddings`

Bitemporal tenant embeddings

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint(20) unsigned` | no | - |  |
| `valid_from` | `date` | no | - |  |
| `valid_to` | `date` | no | - |  |
| `embedding` | `vector(5)` | no | - |  |
| `row_start` | `timestamp(6)` | no | - | `STORED GENERATED`; stored generated as `ROW START`; system-time period start |
| `row_end` | `timestamp(6)` | no | - | `STORED GENERATED`; stored generated as `ROW END`; system-time period end |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `tenant_validity_uq` | `unique` | `tenant_id, row_end, valid_to, valid_from` | period `validity` without overlaps |
| `validity` | `check` | `` | `` `valid_from` < `valid_to` ``; declared at table level |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `embedding_vector_idx` | `embedding` | no | VECTOR; M=8; distance=cosine | - |
| `tenant_validity_uq` | `tenant_id, row_end, valid_to, valid_from` | yes | BTREE; period validity without overlaps | - |


#### MariaDB

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_uca1400_ai_ci`

**System versioning:** enabled

**System-time period:** `row_start, row_end`

**Application-time period:** `validity`: `valid_from` to `valid_to`

```sql
CREATE TABLE `tenant_embeddings` (
  `tenant_id` bigint(20) unsigned NOT NULL,
  `valid_from` date NOT NULL,
  `valid_to` date NOT NULL,
  `embedding` vector(5) NOT NULL,
  `row_start` timestamp(6) GENERATED ALWAYS AS ROW START,
  `row_end` timestamp(6) GENERATED ALWAYS AS ROW END,
  PERIOD FOR `validity` (`valid_from`, `valid_to`),
  UNIQUE KEY `tenant_validity_uq` (`tenant_id`,`row_end`,`validity` WITHOUT OVERLAPS),
  VECTOR KEY `embedding_vector_idx` (`embedding`) `M`=8 `DISTANCE`=cosine,
  PERIOD FOR SYSTEM_TIME (`row_start`, `row_end`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci COMMENT='Bitemporal tenant embeddings' WITH SYSTEM VERSIONING
```


## Views

### `test.active_accounts`

**Check option:** cascaded

**Updatable:** yes

**Security:** invoker

**Algorithm:** merge

**Definer:** `root@localhost`

| Column | Type | Nullable |
|---|---|---|


```sql
CREATE ALGORITHM=MERGE DEFINER=`root`@`localhost` SQL SECURITY INVOKER VIEW `active_accounts` AS select `accounts`.`tenant_id` AS `tenant_id`,`accounts`.`account_id` AS `account_id`,`accounts`.`email` AS `email` from `accounts` where `accounts`.`status` = 'active' WITH CASCADED CHECK OPTION
```

## Triggers

### `test.accounts_changed`

**`after` `insert, update, delete`** on `test.accounts`.

**Orientation:** for each row

**Order:** 1

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` TRIGGER accounts_changed
AFTER INSERT OR UPDATE OR DELETE ON accounts
FOR EACH ROW SET @dbmd_mariadb_account_changed = 1
```

### `test.accounts_updated`

**`before` `update`** on `test.accounts`.

**Orientation:** for each row

**Order:** 1

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Update columns:** `email, status`

```sql
CREATE DEFINER=`root`@`localhost` TRIGGER accounts_updated
BEFORE UPDATE OF status, email ON accounts
FOR EACH ROW SET NEW.status = COALESCE(NEW.status, OLD.status)
```

### `test.accounts_update_marker`

**`before` `update`** on `test.accounts`.

**Orientation:** for each row

**Order:** 2

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` TRIGGER accounts_update_marker
BEFORE UPDATE ON accounts
FOR EACH ROW
SET @dbmd_mariadb_last_account = NEW.account_id
```

## Routines, Sequences, and Events

### `test.disable_account`

**Kind:** procedure

**Data access:** modifies SQL data

**Deterministic:** no

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Parameters:** `in target_id bigint(20) unsigned`

```sql
CREATE DEFINER=`root`@`localhost` PROCEDURE `disable_account`(IN target_id BIGINT UNSIGNED)
    MODIFIES SQL DATA
UPDATE accounts SET status = 'disabled' WHERE account_id = target_id
```


### `test.next_account_id`

**Kind:** procedure

**Data access:** no SQL

**Deterministic:** no

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Parameters:** `in current_id bigint(20) unsigned, out next_id bigint(20) unsigned`

```sql
CREATE DEFINER=`root`@`localhost` PROCEDURE `next_account_id`(IN current_id BIGINT UNSIGNED, OUT next_id BIGINT UNSIGNED)
    NO SQL
SET next_id = current_id + 1
```


### `test.normalize_email`

**Kind:** function

**Data access:** no SQL

**Deterministic:** yes

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Parameters:** `return varchar(255), in value varchar(255) default 'fallback@example.invalid'`

```sql
CREATE DEFINER=`root`@`localhost` FUNCTION `normalize_email`(value VARCHAR(255) DEFAULT 'fallback@example.invalid') RETURNS varchar(255) CHARSET utf8mb4 COLLATE utf8mb4_uca1400_ai_ci
    NO SQL
    DETERMINISTIC
RETURN lower(value)
```


### `test.descending_order_seq`

**Kind:** `sequence`

**Type:** `bigint`

**Start:** `0`

**Minimum:** `-20`

**Maximum:** `0`

**Increment:** `-2`

**Cycle:** yes

**Cache:** `0`

**Engine:** `InnoDB`

```sql
CREATE SEQUENCE `descending_order_seq` start with 0 minvalue -20 maxvalue 0 increment by -2 nocache cycle ENGINE=InnoDB
```


### `test.order_number_seq`

**Kind:** `sequence`

**Type:** `bigint`

**Start:** `1000`

**Minimum:** `1`

**Maximum:** `9223372036854775806`

**Increment:** `10`

**Cycle:** no

**Cache:** `20`

**Engine:** `InnoDB`

```sql
CREATE SEQUENCE `order_number_seq` start with 1000 minvalue 1 maxvalue 9223372036854775806 increment by 10 cache 20 nocycle ENGINE=InnoDB
```


### `test.archive_accounts_once`

**Kind:** `event`

**Status:** disabled

**Schedule:** one time

**Completion:** preserve

**Definer:** `root@localhost`

**Time zone:** `SYSTEM`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Originator:** 1

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Execute at:** `2031-01-01 00:00:00`

```sql
CREATE DEFINER=`root`@`localhost` EVENT `archive_accounts_once` ON SCHEDULE AT '2031-01-01 00:00:00' ON COMPLETION PRESERVE DISABLE DO SET @dbmd_mariadb_archive_requested = 1
```


### `test.purge_disabled_accounts`

Remove old disabled accounts

**Kind:** `event`

**Status:** disabled

**Schedule:** recurring

**Completion:** preserve

**Definer:** `root@localhost`

**Time zone:** `SYSTEM`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Originator:** 1

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Interval:** `1` day

**Starts:** `2030-01-01 00:00:00`

```sql
CREATE DEFINER=`root`@`localhost` EVENT `purge_disabled_accounts` ON SCHEDULE EVERY 1 DAY STARTS '2030-01-01 00:00:00' ON COMPLETION PRESERVE DISABLE COMMENT 'Remove old disabled accounts' DO DELETE FROM accounts WHERE status = 'disabled' AND row_end < CURRENT_TIMESTAMP - INTERVAL 365 DAY
```


### `analytics_remote`

**Kind:** `server`

**Wrapper:** `mariadb`

**Host:** `db.internal`

**Database:** `analytics`

**Username:** `reader`

**Owner:** `platform`

**Port:** `3307`

**Option:** `DATABASE`: `analytics`

**Option:** `HOST`: `db.internal`

**Option:** `OWNER`: `platform`

**Option:** `PASSWORD`: [redacted]

**Option:** `PORT`: `3307`

**Option:** `REGION`: `eu-west-1`

**Option:** `USER`: `reader`



### `Aria (storage engine)`

Crash-safe tables with MyISAM heritage. Used for internal temporary tables and privilege tables

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.6`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.6`

**Author:** `MariaDB Corporation Ab`



### `associative_array (data type)`

Data type ASSOCIATIVE_ARRAY

**Kind:** `plugin`

**Type:** data type

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Rakuten Securities`



### `binlog (daemon)`

This is a plugin to represent the binlog in a transaction

**Kind:** `plugin`

**Type:** daemon

**Version:** `2.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `2.0`

**Author:** `MySQL AB`



### `BLACKHOLE (storage engine)`

/dev/null storage engine (anything you write to it disappears)

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Library:** `ha_blackhole.so`

**Library version:** `1.15`

**Authentication version:** `1.0`

**Author:** `MySQL AB`



### `caching_sha2_password (authentication)`

MySQL-compatible SHA2 authentication

**Kind:** `plugin`

**Type:** authentication

**Version:** `1.0`

**Status:** active

**Type version:** `2.3`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Library:** `auth_mysql_sha2.so`

**Library version:** `1.15`

**Authentication version:** `1.0`

**Author:** `Oracle Corporation, Sergei Golubchik`



### `CLIENT_STATISTICS (information schema)`

Client Statistics

**Kind:** `plugin`

**Type:** information schema

**Version:** `2.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `2.0`

**Author:** `Percona and Sergei Golubchik`



### `CSV (storage engine)`

Stores tables as CSV files

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Brian Aker, MySQL AB`



### `FEEDBACK (information schema)`

MariaDB User Feedback Plugin

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.1`

**Status:** disabled

**Type version:** `120302.0`

**License:** GPL

**Load option:** off

**Maturity:** stable

**Authentication version:** `1.1`

**Author:** `Sergei Golubchik`



### `GEOMETRY_COLUMNS (information schema)`

Lists all geometry columns

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB`



### `INDEX_STATISTICS (information schema)`

Index Statistics

**Kind:** `plugin`

**Type:** information schema

**Version:** `2.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `2.0`

**Author:** `Percona and Sergei Golubchik`



### `inet4 (data type)`

Data type INET4

**Kind:** `plugin`

**Type:** data type

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0.1`

**Author:** `MariaDB Corporation`



### `inet6 (data type)`

Data type INET6

**Kind:** `plugin`

**Type:** data type

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `inet6_aton (native function)`

Function INET6_ATON()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `inet6_ntoa (native function)`

Function INET6_NTOA()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `inet_aton (native function)`

Function INET_ATON()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `inet_ntoa (native function)`

Function INET_NTOA()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `InnoDB (storage engine)`

Supports transactions, row-level locking, foreign keys and encryption for tables

**Kind:** `plugin`

**Type:** storage engine

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_BUFFER_PAGE (information schema)`

InnoDB Buffer Page Information

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_BUFFER_PAGE_LRU (information schema)`

InnoDB Buffer Page in LRU

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_BUFFER_POOL_STATS (information schema)`

InnoDB Buffer Pool Statistics Information

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_CMP (information schema)`

Statistics for the InnoDB compression

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_CMPMEM (information schema)`

Statistics for the InnoDB compressed buffer pool

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_CMPMEM_RESET (information schema)`

Statistics for the InnoDB compressed buffer pool; reset cumulated counts

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_CMP_PER_INDEX (information schema)`

Statistics for the InnoDB compression (per index)

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_CMP_PER_INDEX_RESET (information schema)`

Statistics for the InnoDB compression (per index); reset cumulated counts

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_CMP_RESET (information schema)`

Statistics for the InnoDB compression; reset cumulated counts

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_FT_BEING_DELETED (information schema)`

INNODB AUXILIARY FTS BEING DELETED TABLE

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_FT_CONFIG (information schema)`

INNODB AUXILIARY FTS CONFIG TABLE

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_FT_DEFAULT_STOPWORD (information schema)`

Default stopword list for InnoDB Full Text Search

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_FT_DELETED (information schema)`

INNODB AUXILIARY FTS DELETED TABLE

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_FT_INDEX_CACHE (information schema)`

INNODB AUXILIARY FTS INDEX CACHED

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_FT_INDEX_TABLE (information schema)`

INNODB AUXILIARY FTS INDEX TABLE

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_LOCKS (information schema)`

InnoDB conflicting locks

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_LOCK_WAITS (information schema)`

InnoDB which lock is blocking which

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_METRICS (information schema)`

InnoDB Metrics Info

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_SYS_COLUMNS (information schema)`

InnoDB SYS_COLUMNS

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_SYS_FIELDS (information schema)`

InnoDB SYS_FIELDS

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_SYS_FOREIGN (information schema)`

InnoDB SYS_FOREIGN

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_SYS_FOREIGN_COLS (information schema)`

InnoDB SYS_FOREIGN_COLS

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_SYS_INDEXES (information schema)`

InnoDB SYS_INDEXES

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_SYS_TABLES (information schema)`

InnoDB SYS_TABLES

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_SYS_TABLESPACES (information schema)`

InnoDB tablespaces

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_SYS_TABLESTATS (information schema)`

InnoDB SYS_TABLESTATS

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_SYS_VIRTUAL (information schema)`

InnoDB SYS_VIRTUAL

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `INNODB_TABLESPACES_ENCRYPTION (information schema)`

InnoDB TABLESPACES_ENCRYPTION

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** BSD

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Google Inc`



### `INNODB_TRX (information schema)`

InnoDB transactions

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



### `is_ipv4 (native function)`

Function IS_IPV4()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `is_ipv4_compat (native function)`

Function IS_IPV4_COMPAT()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `is_ipv4_mapped (native function)`

Function IS_IPV4_MAPPED()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `is_ipv6 (native function)`

Function IS_IPV6()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `MEMORY (storage engine)`

Hash based, stored in memory, useful for temporary tables

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MySQL AB`



### `mhnsw (daemon)`

A plugin for mhnsw vector index algorithm

**Kind:** `plugin`

**Type:** daemon

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB plc`



### `MRG_MyISAM (storage engine)`

Collection of identical MyISAM tables

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MySQL AB`



### `MyISAM (storage engine)`

Non-transactional engine with good performance and small data footprint

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MySQL AB`



### `mysql_native_password (authentication)`

Native MySQL authentication

**Kind:** `plugin`

**Type:** authentication

**Version:** `1.0`

**Status:** active

**Type version:** `2.3`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `R.J.Silk, Sergei Golubchik`



### `mysql_old_password (authentication)`

Old MySQL-4.0 authentication

**Kind:** `plugin`

**Type:** authentication

**Version:** `1.0`

**Status:** active

**Type version:** `2.3`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `R.J.Silk, Sergei Golubchik`



### `online_alter_log (daemon)`

This is a plugin to represent the online alter log in a transaction

**Kind:** `plugin`

**Type:** daemon

**Version:** `2.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `2.0`

**Author:** `MariaDB PLC`



### `partition (storage engine)`

Partition Storage Engine Helper

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Mikael Ronstrom, MySQL AB`



### `PERFORMANCE_SCHEMA (storage engine)`

Performance Schema

**Kind:** `plugin`

**Type:** storage engine

**Version:** `0.1`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `5.7.31`

**Author:** `Marc Alff, Oracle`



### `SEQUENCE (storage engine)`

Generated tables filled with sequential values

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `0.1`

**Author:** `Sergei Golubchik`



### `SPATIAL_REF_SYS (information schema)`

Lists all geometry columns

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB`



### `SQL_SEQUENCE (storage engine)`

Sequence Storage Engine for CREATE SEQUENCE

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `jianwei.zhao @ Aliyun & Monty @ MariaDB corp`



### `sys_guid (native function)`

Function SYS_GUID()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `sys_refcursor (data type)`

Data type SYS_REFCURSOR

**Kind:** `plugin`

**Type:** data type

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `TABLE_STATISTICS (information schema)`

Table Statistics

**Kind:** `plugin`

**Type:** information schema

**Version:** `2.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `2.0`

**Author:** `Percona and Sergei Golubchik`



### `THREAD_POOL_GROUPS (information schema)`

Provides information about threadpool groups.

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Vladislav Vaintroub`



### `THREAD_POOL_QUEUES (information schema)`

Provides information about threadpool queues.

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Vladislav Vaintroub`



### `THREAD_POOL_STATS (information schema)`

Provides performance counter information for threadpool.

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Vladislav Vaintroub`



### `THREAD_POOL_WAITS (information schema)`

Provides wait counters for threadpool.

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Vladislav Vaintroub`



### `unix_socket (authentication)`

Unix Socket based authentication

**Kind:** `plugin`

**Type:** authentication

**Version:** `1.1`

**Status:** active

**Type version:** `2.3`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.1`

**Author:** `Sergei Golubchik`



### `USER_STATISTICS (information schema)`

User Statistics

**Kind:** `plugin`

**Type:** information schema

**Version:** `2.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `2.0`

**Author:** `Percona and Sergei Golubchik`



### `user_variables (information schema)`

User-defined variables

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Sergey Vojtovich`



### `uuid (data type)`

Data type UUID

**Kind:** `plugin`

**Type:** data type

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `uuid (native function)`

Function UUID()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `uuid_v4 (native function)`

Function UUID_v4()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0.1`

**Author:** `Stefano Petrilli`



### `uuid_v7 (native function)`

Function UUID_v7()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0.1`

**Author:** `Stefano Petrilli`



### `wsrep (replication)`

Wsrep replication plugin

**Kind:** `plugin`

**Type:** replication

**Version:** `1.0`

**Status:** active

**Type version:** `2.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Codership Oy`



### `WSREP_MEMBERSHIP (information schema)`

Information about group members

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Library:** `wsrep_info.so`

**Library version:** `1.15`

**Authentication version:** `1.0`

**Author:** `Nirbhay Choubey`



### `wsrep_provider (replication)`

Wsrep provider plugin

**Kind:** `plugin`

**Type:** replication

**Version:** `1.0`

**Status:** active

**Type version:** `2.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0.1`

**Author:** `Codership Oy`



### `WSREP_STATUS (information schema)`

Group view information

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Library:** `wsrep_info.so`

**Library version:** `1.15`

**Authentication version:** `1.0`

**Author:** `Nirbhay Choubey`



### `xmltype (data type)`

Data type XMLTYPE

**Kind:** `plugin`

**Type:** data type

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



### `test.analytics_tools`

Analytics package

**Kind:** `package`

**Security:** invoker

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Body:** yes

```sql
CREATE DEFINER=`root`@`localhost` PACKAGE `analytics_tools`    SQL SECURITY INVOKER
    COMMENT 'Analytics package'
 PROCEDURE refresh_cache(IN tenant BIGINT);
FUNCTION normalize(value VARCHAR(255)) RETURNS VARCHAR(255);
END

CREATE DEFINER=`root`@`localhost` PACKAGE BODY `analytics_tools` PROCEDURE refresh_cache(IN tenant BIGINT)
BEGIN
    SELECT tenant;
END;
FUNCTION normalize(value VARCHAR(255)) RETURNS VARCHAR(255)
RETURN lower(value);
END
```


### `analytics_reader@`

**Kind:** `role`

**Host:** ``

**TLS:** none

**Privilege:** `EXECUTE` on `function test.normalize_email`

**Privilege:** `EXECUTE` on `package test.analytics_tools`

**Privilege:** `SELECT` on `schema test`

**Privilege:** `SHOW CREATE ROUTINE` on `schema test`



### `analytics_service@localhost`

**Kind:** `user`

**Host:** `localhost`

**Authentication:** `caching_sha2_password`

**Password lifetime:** 90 days

**Account locked:** yes

**Default role:** `analytics_reader`

**TLS:** specified certificate properties

**TLS cipher:** `TLS_AES_256_GCM_SHA384`

**X.509 issuer:** `/CN=dbmd-ca`

**X.509 subject:** `/CN=dbmd-client`

**Queries per hour:** 17

**Concurrent connections:** 3

**Role:** `analytics_reader` with admin option

**Privilege:** `USAGE` on `global` with grant option

**Privilege:** `PROXY` on `proxy account 'proxy_target'@'localhost'` with grant option



### `healthcheck@127.0.0.1`

**Kind:** `user`

**Host:** `127.0.0.1`

**Authentication:** `mysql_native_password`

**TLS:** none

**Privilege:** `USAGE` on `global`



### `healthcheck@::1`

**Kind:** `user`

**Host:** `::1`

**Authentication:** `mysql_native_password`

**TLS:** none

**Privilege:** `USAGE` on `global`



### `healthcheck@localhost`

**Kind:** `user`

**Host:** `localhost`

**Authentication:** `mysql_native_password`

**TLS:** none

**Privilege:** `USAGE` on `global`



### `mariadb.sys@localhost`

**Kind:** `user`

**Host:** `localhost`

**Authentication:** `mysql_native_password`

**Password expired:** yes

**Account locked:** yes

**TLS:** none

**Privilege:** `USAGE` on `global`



### `proxy_target@localhost`

**Kind:** `user`

**Host:** `localhost`

**Authentication:** `mysql_native_password`

**Account locked:** yes

**TLS:** none

**Privilege:** `USAGE` on `global`



### `root@%`

**Kind:** `user`

**Host:** `%`

**Authentication:** `mysql_native_password`

**TLS:** none

**Privilege:** `ALTER` on `global` with grant option

**Privilege:** `ALTER ROUTINE` on `global` with grant option

**Privilege:** `BINLOG ADMIN` on `global` with grant option

**Privilege:** `BINLOG MONITOR` on `global` with grant option

**Privilege:** `BINLOG REPLAY` on `global` with grant option

**Privilege:** `CONNECTION ADMIN` on `global` with grant option

**Privilege:** `CREATE` on `global` with grant option

**Privilege:** `CREATE ROUTINE` on `global` with grant option

**Privilege:** `CREATE TABLESPACE` on `global` with grant option

**Privilege:** `CREATE TEMPORARY TABLES` on `global` with grant option

**Privilege:** `CREATE USER` on `global` with grant option

**Privilege:** `CREATE VIEW` on `global` with grant option

**Privilege:** `DELETE` on `global` with grant option

**Privilege:** `DELETE HISTORY` on `global` with grant option

**Privilege:** `DROP` on `global` with grant option

**Privilege:** `EVENT` on `global` with grant option

**Privilege:** `EXECUTE` on `global` with grant option

**Privilege:** `FEDERATED ADMIN` on `global` with grant option

**Privilege:** `FILE` on `global` with grant option

**Privilege:** `INDEX` on `global` with grant option

**Privilege:** `INSERT` on `global` with grant option

**Privilege:** `LOCK TABLES` on `global` with grant option

**Privilege:** `PROCESS` on `global` with grant option

**Privilege:** `READ_ONLY ADMIN` on `global` with grant option

**Privilege:** `REFERENCES` on `global` with grant option

**Privilege:** `RELOAD` on `global` with grant option

**Privilege:** `REPLICATION MASTER ADMIN` on `global` with grant option

**Privilege:** `REPLICATION SLAVE` on `global` with grant option

**Privilege:** `REPLICATION SLAVE ADMIN` on `global` with grant option

**Privilege:** `SELECT` on `global` with grant option

**Privilege:** `SET USER` on `global` with grant option

**Privilege:** `SHOW CREATE ROUTINE` on `global` with grant option

**Privilege:** `SHOW DATABASES` on `global` with grant option

**Privilege:** `SHOW VIEW` on `global` with grant option

**Privilege:** `SHUTDOWN` on `global` with grant option

**Privilege:** `SLAVE MONITOR` on `global` with grant option

**Privilege:** `SUPER` on `global` with grant option

**Privilege:** `TRIGGER` on `global` with grant option

**Privilege:** `UPDATE` on `global` with grant option

**Privilege:** `PROXY` on `proxy account ''@'%'` with grant option



### `root@localhost`

**Kind:** `user`

**Host:** `localhost`

**Authentication:** `mysql_native_password`

**TLS:** none

**Role:** `analytics_reader` with admin option

**Privilege:** `ALTER` on `global` with grant option

**Privilege:** `ALTER ROUTINE` on `global` with grant option

**Privilege:** `BINLOG ADMIN` on `global` with grant option

**Privilege:** `BINLOG MONITOR` on `global` with grant option

**Privilege:** `BINLOG REPLAY` on `global` with grant option

**Privilege:** `CONNECTION ADMIN` on `global` with grant option

**Privilege:** `CREATE` on `global` with grant option

**Privilege:** `CREATE ROUTINE` on `global` with grant option

**Privilege:** `CREATE TABLESPACE` on `global` with grant option

**Privilege:** `CREATE TEMPORARY TABLES` on `global` with grant option

**Privilege:** `CREATE USER` on `global` with grant option

**Privilege:** `CREATE VIEW` on `global` with grant option

**Privilege:** `DELETE` on `global` with grant option

**Privilege:** `DELETE HISTORY` on `global` with grant option

**Privilege:** `DROP` on `global` with grant option

**Privilege:** `EVENT` on `global` with grant option

**Privilege:** `EXECUTE` on `global` with grant option

**Privilege:** `FEDERATED ADMIN` on `global` with grant option

**Privilege:** `FILE` on `global` with grant option

**Privilege:** `INDEX` on `global` with grant option

**Privilege:** `INSERT` on `global` with grant option

**Privilege:** `LOCK TABLES` on `global` with grant option

**Privilege:** `PROCESS` on `global` with grant option

**Privilege:** `READ_ONLY ADMIN` on `global` with grant option

**Privilege:** `REFERENCES` on `global` with grant option

**Privilege:** `RELOAD` on `global` with grant option

**Privilege:** `REPLICATION MASTER ADMIN` on `global` with grant option

**Privilege:** `REPLICATION SLAVE` on `global` with grant option

**Privilege:** `REPLICATION SLAVE ADMIN` on `global` with grant option

**Privilege:** `SELECT` on `global` with grant option

**Privilege:** `SET USER` on `global` with grant option

**Privilege:** `SHOW CREATE ROUTINE` on `global` with grant option

**Privilege:** `SHOW DATABASES` on `global` with grant option

**Privilege:** `SHOW VIEW` on `global` with grant option

**Privilege:** `SHUTDOWN` on `global` with grant option

**Privilege:** `SLAVE MONITOR` on `global` with grant option

**Privilege:** `SUPER` on `global` with grant option

**Privilege:** `TRIGGER` on `global` with grant option

**Privilege:** `UPDATE` on `global` with grant option

**Privilege:** `PROXY` on `proxy account ''@''` with grant option



