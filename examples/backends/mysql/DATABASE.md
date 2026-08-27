# Database: `MySQL commerce database`

## Schemas

| Name | Details |
|---|---|
| `test` | Default character set `utf8mb4`; collation `utf8mb4_0900_ai_ci`; encryption no; read-only no. |


## Tables

### `test.accounts`

User accounts

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint unsigned` | no | - |  |
| `account_id` | `bigint unsigned` | no | - | `auto_increment` |
| `email` | `varchar(255)` | no | - |  |
| `normalized_email` | `varchar(255)` | yes | - | `STORED GENERATED`; generated as ``lower(`email`)`` |
| `secret_token` | `varchar(64)` | yes | - | `INVISIBLE`; invisible |
| `status` | `enum('active','disabled')` | no | `active` |  |
| `embedding` | `vector(3)` | yes | - |  |
| `default_embedding` | `vector(2048)` | yes | - |  |
| `home` | `point` | no | - | SRID 4326 |
| `updated_at` | `timestamp` | no | `CURRENT_TIMESTAMP` | `DEFAULT_GENERATED on update CURRENT_TIMESTAMP` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `account_id` | - |
| `accounts_email_check` | `check` | `` | ``(`email` <> _latin1\'\')`` |
| `accounts_status_check` | `check` | `` | ``(`status` in (_latin1\'active\',_latin1\'disabled\'))``; not enforced |
| `accounts_tenant_email_uq` | `unique` | `tenant_id, email` | - |
| `accounts_tenant_fk` | `foreign_key` | `tenant_id` | references `test`.`tenants` (`tenant_id`); on update cascade; on delete restrict |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `account_id` | yes | BTREE | - |
| `accounts_email_desc_idx` | `email DESC` | no | BTREE; invisible; comment `Descending email lookup` | - |
| `accounts_email_ft` | `email` | no | FULLTEXT | - |
| `accounts_home_spatial` | `home(32)` | no | SPATIAL | - |
| `accounts_normalized_idx` | ``lower(`email`)`` | no | BTREE | - |
| `accounts_tenant_email_uq` | `tenant_id, email(120)` | yes | BTREE | - |


#### MySQL

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_0900_ai_ci`

**Create options:** `row_format=DYNAMIC`

```sql
CREATE TABLE `accounts` (
  `tenant_id` bigint unsigned NOT NULL,
  `account_id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `email` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL,
  `normalized_email` varchar(255) GENERATED ALWAYS AS (lower(`email`)) STORED,
  `secret_token` varchar(64) DEFAULT NULL /*!80023 INVISIBLE */,
  `status` enum('active','disabled') NOT NULL DEFAULT 'active',
  `embedding` vector(3) DEFAULT NULL,
  `default_embedding` vector(2048) DEFAULT NULL,
  `home` point NOT NULL /*!80003 SRID 4326 */,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`account_id`),
  UNIQUE KEY `accounts_tenant_email_uq` (`tenant_id`,`email`(120)),
  KEY `accounts_email_desc_idx` (`email` DESC) COMMENT 'Descending email lookup' /*!80000 INVISIBLE */,
  SPATIAL KEY `accounts_home_spatial` (`home`),
  KEY `accounts_normalized_idx` ((lower(`email`))),
  FULLTEXT KEY `accounts_email_ft` (`email`),
  CONSTRAINT `accounts_tenant_fk` FOREIGN KEY (`tenant_id`) REFERENCES `tenants` (`tenant_id`) ON DELETE RESTRICT ON UPDATE CASCADE,
  CONSTRAINT `accounts_email_check` CHECK ((`email` <> _latin1'')),
  CONSTRAINT `accounts_status_check` CHECK ((`status` in (_latin1'active',_latin1'disabled'))) /*!80016 NOT ENFORCED */
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci ROW_FORMAT=DYNAMIC COMMENT='User accounts'
```


### `test.generated_primary_key`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `my_row_id` | `bigint unsigned` | no | - | `auto_increment INVISIBLE`; invisible |
| `payload` | `varchar(64)` | yes | - |  |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `my_row_id` | - |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `my_row_id` | yes | BTREE | - |


#### MySQL

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE TABLE `generated_primary_key` (
  `my_row_id` bigint unsigned NOT NULL AUTO_INCREMENT /*!80023 INVISIBLE */,
  `payload` varchar(64) DEFAULT NULL,
  PRIMARY KEY (`my_row_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci
```


### `test.inline_memberships`

Exercises MySQL 9 inline implicit-parent foreign keys

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `membership_id` | `bigint unsigned` | no | - | `auto_increment` |
| `tenant_id` | `bigint unsigned` | no | - |  |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `membership_id` | - |
| `inline_memberships_ibfk_1` | `foreign_key` | `tenant_id` | references `test`.`tenants` (`tenant_id`); on update no action; on delete cascade |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `membership_id` | yes | BTREE | - |
| `tenant_id` | `tenant_id` | no | BTREE | - |


#### MySQL

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE TABLE `inline_memberships` (
  `membership_id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint unsigned NOT NULL,
  PRIMARY KEY (`membership_id`),
  KEY `tenant_id` (`tenant_id`),
  CONSTRAINT `inline_memberships_ibfk_1` FOREIGN KEY (`tenant_id`) REFERENCES `tenants` (`tenant_id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Exercises MySQL 9 inline implicit-parent foreign keys'
```


### `test.memory_lookup`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `lookup_key` | `varchar(64)` | no | - |  |
| `payload` | `varchar(255)` | yes | - |  |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `lookup_key` | - |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `lookup_key` | yes | HASH | - |
| `memory_payload_hash` | `payload` | no | HASH | - |


#### MySQL

**Engine:** `MEMORY`

**Row format:** `Fixed`

**Collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE TABLE `memory_lookup` (
  `lookup_key` varchar(64) NOT NULL,
  `payload` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`lookup_key`),
  KEY `memory_payload_hash` (`payload`) USING HASH
) ENGINE=MEMORY DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci
```


### `test.monthly_metrics`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `occurred_on` | `date` | no | - |  |
| `metric` | `varchar(64)` | no | - |  |
| `value` | `bigint` | no | - |  |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `occurred_on, metric` | - |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `occurred_on, metric` | yes | BTREE | - |


#### MySQL

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_0900_ai_ci`

**Create options:** `partitioned`

**Subpartition:** `p2025_h0`: method=`HASH`, expression=``month(`occurred_on`)``, boundary=`2026`, position=1; nodegroup=`default`

**Subpartition:** `p2025_h1`: method=`HASH`, expression=``month(`occurred_on`)``, boundary=`2026`, position=2; nodegroup=`default`

**Subpartition:** `pmax_h0`: method=`HASH`, expression=``month(`occurred_on`)``, boundary=`MAXVALUE`, position=1; nodegroup=`default`

**Subpartition:** `pmax_h1`: method=`HASH`, expression=``month(`occurred_on`)``, boundary=`MAXVALUE`, position=2; nodegroup=`default`

```sql
CREATE TABLE `monthly_metrics` (
  `occurred_on` date NOT NULL,
  `metric` varchar(64) NOT NULL,
  `value` bigint NOT NULL,
  PRIMARY KEY (`occurred_on`,`metric`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci
/*!50100 PARTITION BY RANGE (year(`occurred_on`))
SUBPARTITION BY HASH (month(`occurred_on`))
(PARTITION p2025 VALUES LESS THAN (2026)
 (SUBPARTITION p2025_h0 ENGINE = InnoDB,
  SUBPARTITION p2025_h1 ENGINE = InnoDB),
 PARTITION pmax VALUES LESS THAN MAXVALUE
 (SUBPARTITION pmax_h0 ENGINE = InnoDB,
  SUBPARTITION pmax_h1 ENGINE = InnoDB)) */
```


### `test.tenants`

Application tenants

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint unsigned` | no | - | `auto_increment` |
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


#### MySQL

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE TABLE `tenants` (
  `tenant_id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(120) NOT NULL COMMENT 'Tenant display name',
  PRIMARY KEY (`tenant_id`),
  UNIQUE KEY `tenants_name_uq` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Application tenants'
```


## Views

### `test.active_accounts`

**Kind:** `sql`

**Check option:** cascaded

**Updatable:** yes

**Security:** invoker

**Definer:** `root@localhost`

| Column | Type | Nullable |
|---|---|---|


```sql
CREATE ALGORITHM=UNDEFINED DEFINER=`root`@`localhost` SQL SECURITY INVOKER VIEW `active_accounts` AS select `accounts`.`tenant_id` AS `tenant_id`,`accounts`.`account_id` AS `account_id`,`accounts`.`email` AS `email` from `accounts` where (`accounts`.`status` = 'active') WITH CASCADED CHECK OPTION
```

### `test.tenant_documents`

**Kind:** `json_relational_duality`

**Check option:** none

**Updatable:** yes

**Security:** definer

**Definer:** `root@localhost`

**JSON column:** `data`

**Root table:** `test.tenants`

**Status:** `valid`

**Operations:** `insert=false, update=false, delete=false, read_only=true`

**Mapped table:** `#0 test.tenants parent=None relationship=- where=- permissions=false/false/false read_only=true root=true`

**JSON field:** `_id -> #0 test.tenants.tenant_id, permissions=false/false/false read_only=true root=true`

**JSON field:** `name -> #0 test.tenants.name, permissions=false/false/false read_only=true root=true`

| Column | Type | Nullable |
|---|---|---|


```sql
CREATE ALGORITHM=UNDEFINED DEFINER=`root`@`localhost` SQL SECURITY DEFINER JSON RELATIONAL DUALITY VIEW `tenant_documents` AS select json_duality_object('_id':`tenants`.`tenant_id`,'name':`tenants`.`name`) AS `JSON_DUALITY_OBJECT('_id':tenant_id, 'name':name)` from `tenants`
```

## Triggers

### `test.accounts_updated`

**`before` `update`** on `test.accounts`.

**Orientation:** for each row

**Order:** 1

**Definer:** `root@localhost`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` TRIGGER `accounts_updated` BEFORE UPDATE ON `accounts` FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP
```

### `test.accounts_update_marker`

**`before` `update`** on `test.accounts`.

**Orientation:** for each row

**Order:** 2

**Definer:** `root@localhost`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` TRIGGER `accounts_update_marker` BEFORE UPDATE ON `accounts` FOR EACH ROW SET @dbmd_last_account = NEW.account_id
```

## Routines

### `test.disable_account`

**Kind:** procedure

**Data access:** modifies SQL data

**Deterministic:** no

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

**Parameters:** `in target_id bigint unsigned`

**External language:** `SQL`

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

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

**Parameters:** `in current_id bigint unsigned, out next_id bigint unsigned`

**External language:** `SQL`

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

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

**Parameters:** `return varchar(255), in value varchar(255)`

**External language:** `SQL`

```sql
CREATE DEFINER=`root`@`localhost` FUNCTION `normalize_email`(value VARCHAR(255)) RETURNS varchar(255) CHARSET utf8mb4
    NO SQL
    DETERMINISTIC
RETURN lower(value)
```


## Events

### `test.archive_accounts_once`

**Definer:** `root@localhost`

**Type:** one time

**Status:** disabled

**Time zone:** `SYSTEM`

**On completion:** preserve

**Schedule:** `AT 2031-01-01 00:00:00`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Originator server ID:** 1

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` EVENT `archive_accounts_once` ON SCHEDULE AT '2031-01-01 00:00:00' ON COMPLETION PRESERVE DISABLE DO SET @dbmd_archive_requested = 1
```


### `test.purge_disabled_accounts`

Remove old disabled accounts

**Definer:** `root@localhost`

**Type:** recurring

**Status:** disabled

**Time zone:** `SYSTEM`

**On completion:** preserve

**Schedule:** `EVERY 1 DAY STARTS 2030-01-01 00:00:00`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Originator server ID:** 1

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` EVENT `purge_disabled_accounts` ON SCHEDULE EVERY 1 DAY STARTS '2030-01-01 00:00:00' ON COMPLETION PRESERVE DISABLE COMMENT 'Remove old disabled accounts' DO DELETE FROM accounts WHERE status = 'disabled' AND updated_at < CURRENT_TIMESTAMP - INTERVAL 365 DAY
```


