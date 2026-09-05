# `tenancy.special_events`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain`; inherited only |
| `payload` | `jsonb` | no | - | storage `extended`; inherited only |
| `category` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `base_events_payload_not_null` | `not_null` | `payload` | `NOT NULL payload`; inherited |
| `base_events_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id`; inherited |
| `special_events_category_not_null` | `not_null` | `category` | `NOT NULL category` |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Inherits:** `tenancy.base_events`

