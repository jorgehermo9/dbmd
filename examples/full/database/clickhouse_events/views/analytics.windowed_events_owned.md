# `analytics.windowed_events_owned`

**Kind:** `window_view`

**UUID:** `20000000-0000-0000-0000-000000000020`

**Window inner engine:** `AggregatingMergeTree ORDER BY tuple()`

**Window storage engine:** `MergeTree ORDER BY window_end SETTINGS index_granularity = 8192`

**Watermark:** `ASCENDING`

**Allowed lateness:** `toIntervalSecond('2')`

**AS SELECT:** `SELECT count(event_id) AS total, tumbleEnd(window_id) AS window_end FROM analytics.retention_matrix GROUP BY tumble(occurred_at, toIntervalSecond('5')) AS window_id`

| Column | Type | Nullable |
|---|---|---|
| `total` | `UInt64` | no |
| `window_end` | `DateTime` | no |


```sql
CREATE WINDOW VIEW analytics.windowed_events_owned (`total` UInt64, `window_end` DateTime) INNER ENGINE = AggregatingMergeTree ORDER BY tuple() ENGINE = MergeTree ORDER BY window_end SETTINGS index_granularity = 8192 WATERMARK ASCENDING ALLOWED_LATENESS toIntervalSecond('2') AS SELECT count(event_id) AS total, tumbleEnd(window_id) AS window_end FROM analytics.retention_matrix GROUP BY tumble(occurred_at, toIntervalSecond('5')) AS window_id
```