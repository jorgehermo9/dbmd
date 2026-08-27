# `test.memory_lookup`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `lookup_key` | `varchar(64)` | no | - |  |
| `payload` | `varchar(255)` | yes | - |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `lookup_key` | - |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `lookup_key` | yes | HASH | - |
| `memory_payload_hash` | `payload` | no | HASH | - |


## MySQL

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
