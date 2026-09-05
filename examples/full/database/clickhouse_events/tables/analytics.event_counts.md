# `analytics.event_counts`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `event_type` | `LowCardinality(String)` | no | - | statistics `Uniq(auto)`; keys `primary, sorting` |
| `total` | `AggregateFunction(count)` | no | - |  |


## ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000002`

**Engine:** `AggregatingMergeTree ORDER BY event_type SETTINGS index_granularity = 8192`

**Primary key:** `event_type`

**Sorting key:** `event_type`

**Storage policy:** `default`

**Setting:** `index_granularity` = `8192`

```sql
CREATE TABLE analytics.event_counts (`event_type` LowCardinality(String), `total` AggregateFunction(count)) ENGINE = AggregatingMergeTree ORDER BY event_type SETTINGS index_granularity = 8192
```
