# `analytics.active_events`

Non-deleted events

**Kind:** `view`

**UUID:** `20000000-0000-0000-0000-000000000004`

**Definer:** `default`

**SQL security:** `invoker`

**AS SELECT:** `SELECT tenant_id, event_id, occurred_at, event_type FROM analytics.events WHERE deleted = 0`

| Column | Type | Nullable |
|---|---|---|
| `tenant_id` | `UInt32` | no |
| `event_id` | `UUID` | no |
| `occurred_at` | `DateTime64(3, 'UTC')` | no |
| `event_type` | `LowCardinality(String)` | no |


```sql
CREATE VIEW analytics.active_events (`tenant_id` UInt32, `event_id` UUID, `occurred_at` DateTime64(3, 'UTC'), `event_type` LowCardinality(String)) DEFINER = default SQL SECURITY INVOKER COMMENT 'Non-deleted events' AS SELECT tenant_id, event_id, occurred_at, event_type FROM analytics.events WHERE deleted = 0
```