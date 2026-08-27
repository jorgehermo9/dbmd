# `analytics.retention_rollup`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0; keys `primary, sorting` |
| `occurred_at` | `DateTime` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; datetime precision 0 |
| `amount` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0 |


## ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000009`

**Engine:** `MergeTree ORDER BY tenant_id TTL occurred_at + toIntervalDay(30) GROUP BY tenant_id SET amount = sum(amount) SETTINGS index_granularity = 8192`

**Primary key:** `tenant_id`

**Sorting key:** `tenant_id`

**Storage policy:** `default`

**TTL:** `occurred_at + toIntervalDay(30)`; `group by tenant_id set amount = sum(amount)`

**Setting:** `index_granularity` = `8192`

```sql
CREATE TABLE analytics.retention_rollup (`tenant_id` UInt64, `occurred_at` DateTime, `amount` UInt64) ENGINE = MergeTree ORDER BY tenant_id TTL occurred_at + toIntervalDay(30) GROUP BY tenant_id SET amount = sum(amount) SETTINGS index_granularity = 8192
```
