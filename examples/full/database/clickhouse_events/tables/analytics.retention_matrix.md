# `analytics.retention_matrix`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `event_id` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0; keys `primary, sorting` |
| `occurred_at` | `DateTime` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; datetime precision 0 |
| `expires_at` | `DateTime` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; TTL `expires_at + toIntervalDay(1)`; datetime precision 0 |
| `deleted` | `UInt8` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 8 base 2; scale 0 |
| `payload` | `String` | no | - | serialization `Default`; statistics `Uniq(auto)` |


## ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000008`

**Engine:** `MergeTree ORDER BY event_id TTL occurred_at + toIntervalDay(7) TO DISK 'default', occurred_at + toIntervalDay(10) TO VOLUME 'default', occurred_at + toIntervalDay(14) RECOMPRESS CODEC(ZSTD(9)), occurred_at + toIntervalDay(30) WHERE deleted = 1 SETTINGS index_granularity = 4096`

**Primary key:** `event_id`

**Sorting key:** `event_id`

**Storage policy:** `default`

**TTL:** `occurred_at + toIntervalDay(7)`; `move to disk 'default'`

**TTL:** `occurred_at + toIntervalDay(10)`; `move to volume 'default'`

**TTL:** `occurred_at + toIntervalDay(14)`; `recompress CODEC(ZSTD(9))`

**TTL:** `occurred_at + toIntervalDay(30)`; `delete where deleted = 1`

**Setting:** `index_granularity` = `4096`

```sql
CREATE TABLE analytics.retention_matrix (`event_id` UInt64, `occurred_at` DateTime, `expires_at` DateTime TTL expires_at + toIntervalDay(1), `deleted` UInt8, `payload` String) ENGINE = MergeTree ORDER BY event_id TTL occurred_at + toIntervalDay(7) TO DISK 'default', occurred_at + toIntervalDay(10) TO VOLUME 'default', occurred_at + toIntervalDay(14) RECOMPRESS CODEC(ZSTD(9)), occurred_at + toIntervalDay(30) WHERE deleted = 1 SETTINGS index_granularity = 4096
```
