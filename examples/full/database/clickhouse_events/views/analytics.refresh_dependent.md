# `analytics.refresh_dependent`

**Kind:** `materialized_view`

**UUID:** `20000000-0000-0000-0000-000000000013`

**Target:** `analytics.refresh_rollups`

**Refresh:** `after 2 HOUR`

**Refresh mode:** `replace`

**Refresh depends on:** `analytics.refresh_base`

**Refresh setting:** `refresh_retries` = `3`

**Definer:** `default`

**SQL security:** `definer`

**AS SELECT:** `SELECT tenant_id, count() AS total FROM analytics.events GROUP BY tenant_id`

| Column | Type | Nullable |
|---|---|---|
| `tenant_id` | `UInt32` | no |
| `total` | `UInt64` | no |


```sql
CREATE MATERIALIZED VIEW analytics.refresh_dependent REFRESH AFTER 2 HOUR DEPENDS ON analytics.refresh_base SETTINGS refresh_retries = 3 TO analytics.refresh_rollups (`tenant_id` UInt32, `total` UInt64) DEFINER = default SQL SECURITY DEFINER AS SELECT tenant_id, count() AS total FROM analytics.events GROUP BY tenant_id
```