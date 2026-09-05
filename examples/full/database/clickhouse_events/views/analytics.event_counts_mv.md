# `analytics.event_counts_mv`

**Kind:** `materialized_view`

**UUID:** `20000000-0000-0000-0000-000000000003`

**Target:** `analytics.event_counts`

**AS SELECT:** `SELECT event_type, countState() AS total FROM analytics.events GROUP BY event_type`

| Column | Type | Nullable |
|---|---|---|
| `event_type` | `LowCardinality(String)` | no |
| `total` | `AggregateFunction(count)` | no |


```sql
CREATE MATERIALIZED VIEW analytics.event_counts_mv TO analytics.event_counts (`event_type` LowCardinality(String), `total` AggregateFunction(count)) AS SELECT event_type, countState() AS total FROM analytics.events GROUP BY event_type
```