# `analytics.window_event_counts`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `total` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0 |
| `window_end` | `DateTime` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; datetime precision 0; keys `primary, sorting` |


## ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000018`

**Engine:** `MergeTree ORDER BY window_end SETTINGS index_granularity = 8192`

**Primary key:** `window_end`

**Sorting key:** `window_end`

**Storage policy:** `default`

**Setting:** `index_granularity` = `8192`

```sql
CREATE TABLE analytics.window_event_counts (`total` UInt64, `window_end` DateTime) ENGINE = MergeTree ORDER BY window_end SETTINGS index_granularity = 8192
```
