# `tenancy.base_events`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `payload` | `jsonb` | no | - | storage `extended` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `base_events_payload_not_null` | `not_null` | `payload` | `NOT NULL payload` |
| `base_events_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Policy:** `tenant_events` `select` to `PUBLIC` (restrictive); using `tenant_id = current_setting('app.tenant_id'::text)::bigint`; Restricts events to the active tenant

Row-level security enabled.

Row-level security forced for the table owner.

