# `analytics.refresh_base`

**Kind:** `materialized_view`

**UUID:** `20000000-0000-0000-0000-000000000012`

**Target:** `analytics.refresh_snapshots`

**Refresh:** `every 1 HOUR`

**Refresh offset:** `5 MINUTE`

**Refresh randomization:** `1 MINUTE`

**Refresh mode:** `append`

**Refresh setting:** `refresh_retries` = `5`

**Definer:** `default`

**SQL security:** `definer`

**AS SELECT:** `SELECT tenant_id, now() AS captured_at FROM analytics.events`

| Column | Type | Nullable |
|---|---|---|
| `tenant_id` | `UInt32` | no |
| `captured_at` | `DateTime` | no |


```sql
CREATE MATERIALIZED VIEW analytics.refresh_base REFRESH EVERY 1 HOUR OFFSET 5 MINUTE RANDOMIZE FOR 1 MINUTE SETTINGS refresh_retries = 5 APPEND TO analytics.refresh_snapshots (`tenant_id` UInt32, `captured_at` DateTime) DEFINER = default SQL SECURITY DEFINER AS SELECT tenant_id, now() AS captured_at FROM analytics.events
```