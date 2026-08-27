# `tenancy.events`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `created_at` | `date` | no | - | storage `plain` |
| `payload` | `jsonb` | no | - | storage `extended` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `events_created_at_not_null` | `not_null` | `created_at` | `NOT NULL created_at` |
| `events_payload_not_null` | `not_null` | `payload` | `NOT NULL payload` |
| `events_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `events_created_idx` | `created_at` ascending opclass `pg_catalog.date_ops` nulls `last` | no | postgres `btree`; owner `dbmd`; partitioned | - |


## PostgreSQL

**Kind:** `partitioned_table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Partition key:** `RANGE (created_at)`

