# `test.discarded_events`

Exercises an installed storage-engine plugin

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `event_id` | `bigint(20)` | no | - |  |


## MariaDB

**Engine:** `BLACKHOLE`

**Row format:** `Fixed`

**Collation:** `utf8mb4_uca1400_ai_ci`

```sql
CREATE TABLE `discarded_events` (
  `event_id` bigint(20) NOT NULL
) ENGINE=BLACKHOLE DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci COMMENT='Exercises an installed storage-engine plugin'
```
