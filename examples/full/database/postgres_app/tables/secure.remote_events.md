# `secure.remote_events`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | yes | - | storage `plain` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


## PostgreSQL

**Kind:** `foreign_table`

**Owner:** `dbmd_acl_owner`

**Persistence:** `permanent`

**Replica identity:** `nothing`

**Foreign server:** `secure_server`

**Foreign-data wrapper:** `postgres_fdw`

**Foreign option:** `table_name=events`

