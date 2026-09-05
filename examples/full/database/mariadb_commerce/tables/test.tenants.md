# `test.tenants`

Application tenants

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint(20) unsigned` | no | - | `auto_increment` |
| `name` | `varchar(120)` | no | - | Tenant display name |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `tenant_id` | - |
| `tenants_name_uq` | `unique` | `name` | - |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `tenant_id` | yes | BTREE | - |
| `tenants_name_uq` | `name` | yes | BTREE | - |


## MariaDB

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
