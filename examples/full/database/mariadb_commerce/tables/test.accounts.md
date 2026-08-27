# `test.accounts`

Versioned user accounts

## Columns

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


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `account_id, row_end` | - |
| `accounts_email_check` | `check` | `` | `` `email` <> '' ``; declared at table level |
| `accounts_tenant_email_uq` | `unique` | `tenant_id, email, row_end` | - |
| `accounts_tenant_fk` | `foreign_key` | `tenant_id` | references `test`.`tenants` (`tenant_id`); on update cascade; on delete restrict |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `account_id, row_end` | yes | BTREE | - |
| `accounts_email_fulltext` | `email` | no | FULLTEXT | - |
| `accounts_email_ignored_idx` | `email` | no | BTREE; ignored | - |
| `accounts_home_spatial` | `home(32)` | no | SPATIAL | - |
| `accounts_status_desc_idx` | `status DESC` | no | BTREE; comment `Status lookup ordering` | - |
| `accounts_tenant_email_uq` | `tenant_id, email(120), row_end` | yes | BTREE | - |


## MariaDB

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
