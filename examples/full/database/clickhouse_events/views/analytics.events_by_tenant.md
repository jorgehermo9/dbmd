# `analytics.events_by_tenant`

**Kind:** `view`

**UUID:** `20000000-0000-0000-0000-000000000017`

**AS SELECT:** `SELECT tenant_id, event_id, occurred_at, event_type FROM analytics.events WHERE tenant_id = {requested_tenant:UInt32}`

**Parameter:** `requested_tenant` `UInt32`

| Column | Type | Nullable |
|---|---|---|


```sql
CREATE VIEW analytics.events_by_tenant AS SELECT tenant_id, event_id, occurred_at, event_type FROM analytics.events WHERE tenant_id = {requested_tenant:UInt32}
```