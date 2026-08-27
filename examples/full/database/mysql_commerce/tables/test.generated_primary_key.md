# `test.generated_primary_key`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `my_row_id` | `bigint unsigned` | no | - | `auto_increment INVISIBLE`; invisible |
| `payload` | `varchar(64)` | yes | - |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `my_row_id` | - |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `my_row_id` | yes | BTREE | - |


## MySQL

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
