# `storage.remote_events`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `event_id` | `bigint` | yes | - | storage `plain`; foreign option `remote_name=external_id` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


## PostgreSQL

**Kind:** `foreign_table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `nothing`

**Foreign server:** `fixture_server`

**Foreign-data wrapper:** `fixture_wrapper`

**Foreign option:** `schema_name=remote`

**Foreign option:** `table_name=events`

