# `analytics.country_source`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `country_id` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0; keys `primary, sorting` |
| `parent_id` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0 |
| `country_name` | `String` | no | - | serialization `Default`; statistics `Uniq(auto)` |
| `valid_from` | `Date` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; datetime precision 0 |
| `valid_to` | `Date` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; datetime precision 0 |
| `rate` | `Float64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)` |


## ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000005`

**Engine:** `MergeTree ORDER BY country_id SETTINGS index_granularity = 8192`

**Primary key:** `country_id`

**Sorting key:** `country_id`

**Storage policy:** `default`

**Setting:** `index_granularity` = `8192`

**Loads before:** `analytics.country_names`

**Loads before:** `analytics.country_rates`

```sql
CREATE TABLE analytics.country_source (`country_id` UInt64, `parent_id` UInt64, `country_name` String, `valid_from` Date, `valid_to` Date, `rate` Float64) ENGINE = MergeTree ORDER BY country_id SETTINGS index_granularity = 8192
```
