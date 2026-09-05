# `tenancy.events_2025`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain`; inherited only |
| `created_at` | `date` | no | - | storage `plain`; inherited only |
| `payload` | `jsonb` | no | - | storage `extended`; inherited only |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `events_created_at_not_null` | `not_null` | `created_at` | `NOT NULL created_at`; inherited |
| `events_payload_not_null` | `not_null` | `payload` | `NOT NULL payload`; inherited |
| `events_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id`; inherited |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `events_2025_created_idx` | `created_at` ascending opclass `pg_catalog.date_ops` nulls `last` | no | postgres `btree`; owner `dbmd`; option `fillfactor=76`; parent `tenancy.events_created_idx` | - |


## PostgreSQL

**Kind:** `partition`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Inherits:** `tenancy.events`

**Partition parent:** `tenancy.events`

**Partition bound:** `FOR VALUES FROM ('2025-01-01') TO ('2026-01-01')`

