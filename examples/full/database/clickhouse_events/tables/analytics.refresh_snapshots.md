# `analytics.refresh_snapshots`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `UInt32` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 32 base 2; scale 0; keys `primary, sorting` |
| `captured_at` | `DateTime` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; datetime precision 0 |


## ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000010`

**Engine:** `MergeTree ORDER BY tenant_id SETTINGS index_granularity = 8192`

**Primary key:** `tenant_id`

**Sorting key:** `tenant_id`

**Storage policy:** `default`

**Setting:** `index_granularity` = `8192`

```sql
CREATE TABLE analytics.refresh_snapshots (`tenant_id` UInt32, `captured_at` DateTime) ENGINE = MergeTree ORDER BY tenant_id SETTINGS index_granularity = 8192
```
