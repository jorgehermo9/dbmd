# `test.inline_memberships`

Exercises MySQL 9 inline implicit-parent foreign keys

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `membership_id` | `bigint unsigned` | no | - | `auto_increment` |
| `tenant_id` | `bigint unsigned` | no | - |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `membership_id` | - |
| `inline_memberships_ibfk_1` | `foreign_key` | `tenant_id` | references `test`.`tenants` (`tenant_id`); on update no action; on delete cascade |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `membership_id` | yes | BTREE | - |
| `tenant_id` | `tenant_id` | no | BTREE | - |


## MySQL

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
