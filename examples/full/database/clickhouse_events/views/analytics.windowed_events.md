# `analytics.windowed_events`

**Kind:** `window_view`

**UUID:** `20000000-0000-0000-0000-000000000019`

**Target:** `analytics.window_event_counts`

**Window inner engine:** `AggregatingMergeTree ORDER BY tuple()`

**Watermark:** `STRICTLY_ASCENDING`

**AS SELECT:** `SELECT count(event_id) AS total, tumbleEnd(window_id) AS window_end FROM analytics.retention_matrix GROUP BY tumble(occurred_at, toIntervalSecond('5')) AS window_id`

| Column | Type | Nullable |
|---|---|---|
| `total` | `UInt64` | no |
| `window_end` | `DateTime` | no |


```sql
CREATE WINDOW VIEW analytics.windowed_events TO analytics.window_event_counts (`total` UInt64, `window_end` DateTime) INNER ENGINE = AggregatingMergeTree ORDER BY tuple() WATERMARK STRICTLY_ASCENDING AS SELECT count(event_id) AS total, tumbleEnd(window_id) AS window_end FROM analytics.retention_matrix GROUP BY tumble(occurred_at, toIntervalSecond('5')) AS window_id
```