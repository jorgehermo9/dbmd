# `secure.events`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `events_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `events_pkey` | `primary_key` | `id` | `PRIMARY KEY (id)`; no inherit |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `events_pkey` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd_acl_owner`; constraint `events_pkey` | - |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd_acl_owner`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

