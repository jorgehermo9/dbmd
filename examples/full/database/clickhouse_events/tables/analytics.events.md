# `analytics.events`

Immutable analytical events

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `UInt32` | no | - | Owning tenant; serialization `Default`; statistics `minmax(auto)`; precision 32 base 2; scale 0; keys `primary, sorting` |
| `event_id` | `UUID` | no | - | serialization `Default`; keys `primary, sorting` |
| `occurred_at` | `DateTime64(3, 'UTC')` | no | - | codec `CODEC(DoubleDelta, ZSTD(1))`; serialization `Default`; statistics `minmax(auto)`; datetime precision 3; keys `partition, sorting` |
| `event_type` | `LowCardinality(String)` | no | `'unknown'` | `default` expression |
| `payload` | `String` | no | - | codec `CODEC(ZSTD(3))`; serialization `Default` |
| `vector` | `QBit(Float32, 8)` | no | - |  |
| `expires_at` | `DateTime` | no | `toDateTime(occurred_at) + toIntervalDay(30)` | serialization `Default`; statistics `minmax(auto)`; datetime precision 0; `materialized` expression |
| `version` | `UInt64` | no | - | serialization `Default`; statistics `minmax(auto)`; precision 64 base 2; scale 0 |
| `deleted` | `UInt8` | no | `0` | serialization `Default`; statistics `minmax(auto)`; precision 8 base 2; scale 0; `default` expression |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `positive_tenant` | `assumption` | - | `tenant_id > 0` |
| `valid_deleted` | `check` | - | `deleted IN (0, 1)` |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `auto_minmax_index_expires_at` | `expires_at` | no | clickhouse `minmax()` granularity 1; implicit | - |
| `auto_minmax_index_occurred_at` | `occurred_at` | no | clickhouse `minmax()` granularity 1; implicit | - |
| `payload_text` | `payload` | no | clickhouse `text(tokenizer = 'splitByNonAlpha')` granularity 100000000 | - |
| `payload_tokens` | `lower(payload)` | no | clickhouse `tokenbf_v1(1024, 3, 0)` granularity 4 | - |


## ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000001`

**Engine:** `ReplacingMergeTree(version, deleted) PARTITION BY toYYYYMM(occurred_at) PRIMARY KEY (tenant_id, event_id) ORDER BY (tenant_id, event_id, occurred_at) TTL toDateTime(occurred_at) + toIntervalDay(365) SETTINGS index_granularity = 4096, deduplicate_merge_projection_mode = 'drop', auto_statistics_types = 'minmax', add_minmax_index_for_temporal_columns = 1`

**Engine argument:** 1: `version`

**Engine argument:** 2: `deleted`

**Partition key:** `toYYYYMM(occurred_at)`

**Primary key:** `tenant_id, event_id`

**Sorting key:** `tenant_id, event_id, occurred_at`

**Storage policy:** `default`

**TTL:** `toDateTime(occurred_at) + toIntervalDay(365)`; `delete`

**Setting:** `add_minmax_index_for_temporal_columns` = `1`

**Setting:** `auto_statistics_types` = `'minmax'`

**Setting:** `deduplicate_merge_projection_mode` = `'drop'`

**Setting:** `index_granularity` = `4096`

**Depends on:** `analytics.event_counts_mv`

**Projection:** `by_event_type` `Aggregate sorted by event_type`: `SELECT event_type, count() GROUP BY event_type`

**Projection:** `by_time` `index occurred_at type basic`: `occurred_at`

```sql
CREATE TABLE analytics.events (`tenant_id` UInt32 COMMENT 'Owning tenant', `event_id` UUID, `occurred_at` DateTime64(3, 'UTC') CODEC(DoubleDelta, ZSTD(1)), `event_type` LowCardinality(String) DEFAULT 'unknown', `payload` String CODEC(ZSTD(3)), `vector` QBit(Float32, 8), `expires_at` DateTime MATERIALIZED toDateTime(occurred_at) + toIntervalDay(30), `version` UInt64, `deleted` UInt8 DEFAULT 0, INDEX payload_tokens lower(payload) TYPE tokenbf_v1(1024, 3, 0) GRANULARITY 4, INDEX payload_text payload TYPE text(tokenizer = 'splitByNonAlpha') GRANULARITY 100000000, CONSTRAINT valid_deleted CHECK deleted IN (0, 1), CONSTRAINT positive_tenant ASSUME tenant_id > 0, PROJECTION by_event_type (SELECT event_type, count() GROUP BY event_type), PROJECTION by_time INDEX occurred_at TYPE basic) ENGINE = ReplacingMergeTree(version, deleted) PARTITION BY toYYYYMM(occurred_at) PRIMARY KEY (tenant_id, event_id) ORDER BY (tenant_id, event_id, occurred_at) TTL toDateTime(occurred_at) + toIntervalDay(365) SETTINGS index_granularity = 4096, deduplicate_merge_projection_mode = 'drop', auto_statistics_types = 'minmax', add_minmax_index_for_temporal_columns = 1 COMMENT 'Immutable analytical events'
```
