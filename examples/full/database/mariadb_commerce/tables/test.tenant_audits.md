# `test.tenant_audits`

Reuses a foreign-key name in the same schema

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `audit_id` | `bigint(20) unsigned` | no | - | `auto_increment` |
| `tenant_id` | `bigint(20) unsigned` | no | - |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `audit_id` | - |
| `accounts_tenant_fk` | `foreign_key` | `tenant_id` | references `test`.`tenants` (`tenant_id`); on update restrict; on delete cascade |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `audit_id` | yes | BTREE | - |
| `accounts_tenant_fk` | `tenant_id` | no | BTREE | - |


## MariaDB

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
