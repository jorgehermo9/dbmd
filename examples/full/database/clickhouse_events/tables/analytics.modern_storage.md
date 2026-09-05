# `analytics.modern_storage`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0; keys `primary, sorting` |
| `measurement` | `Float64` | no | - | codec `CODEC(ALP, ZSTD(1))`; serialization `Default`; statistics `Uniq(auto),minmax(auto)` |
| `attributes` | `Map(String, String)` | no | - |  |


## ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000014`

**Engine:** `CoalescingMergeTree ORDER BY id SETTINGS map_serialization_version = 'with_buckets', map_serialization_version_for_zero_level_parts = 'basic', map_buckets_strategy = 'linear', map_buckets_coefficient = 0.5, map_buckets_min_avg_size = 0, max_buckets_in_map = 64, index_granularity = 8192`

**Primary key:** `id`

**Sorting key:** `id`

**Storage policy:** `default`

**Setting:** `index_granularity` = `8192`

**Setting:** `map_buckets_coefficient` = `0.5`

**Setting:** `map_buckets_min_avg_size` = `0`

**Setting:** `map_buckets_strategy` = `'linear'`

**Setting:** `map_serialization_version` = `'with_buckets'`

**Setting:** `map_serialization_version_for_zero_level_parts` = `'basic'`

**Setting:** `max_buckets_in_map` = `64`

```sql
CREATE TABLE analytics.modern_storage (`id` UInt64, `measurement` Float64 CODEC(ALP, ZSTD(1)), `attributes` Map(String, String)) ENGINE = CoalescingMergeTree ORDER BY id SETTINGS map_serialization_version = 'with_buckets', map_serialization_version_for_zero_level_parts = 'basic', map_buckets_strategy = 'linear', map_buckets_coefficient = 0.5, map_buckets_min_avg_size = 0, max_buckets_in_map = 64, index_granularity = 8192
```
