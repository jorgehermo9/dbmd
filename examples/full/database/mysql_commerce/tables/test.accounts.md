# `test.accounts`

User accounts

## Columns

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


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `account_id` | - |
| `accounts_email_check` | `check` | `` | ``(`email` <> _latin1\'\')`` |
| `accounts_status_check` | `check` | `` | ``(`status` in (_latin1\'active\',_latin1\'disabled\'))``; not enforced |
| `accounts_tenant_email_uq` | `unique` | `tenant_id, email` | - |
| `accounts_tenant_fk` | `foreign_key` | `tenant_id` | references `test`.`tenants` (`tenant_id`); on update cascade; on delete restrict |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `account_id` | yes | BTREE | - |
| `accounts_email_desc_idx` | `email DESC` | no | BTREE; invisible; comment `Descending email lookup` | - |
| `accounts_email_ft` | `email` | no | FULLTEXT | - |
| `accounts_home_spatial` | `home(32)` | no | SPATIAL | - |
| `accounts_normalized_idx` | ``lower(`email`)`` | no | BTREE | - |
| `accounts_tenant_email_uq` | `tenant_id, email(120)` | yes | BTREE | - |


## MySQL

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
