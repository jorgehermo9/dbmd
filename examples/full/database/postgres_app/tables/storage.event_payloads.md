# `storage.event_payloads`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `event_id` | `bigint` | no | - | storage `plain` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `external`; compression `lz4`; statistics target 777; option `n_distinct=-0.5` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `event_payloads_event_id_not_null` | `not_null` | `event_id` | `NOT NULL event_id` |
| `event_payloads_pkey` | `primary_key` | `event_id` | `PRIMARY KEY (event_id)`; no inherit |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `event_payloads_pkey` | `event_id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `event_payloads_pkey` | - |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `unlogged`

**Replica identity:** `full`

**Access method:** `heap`

**Option:** `fillfactor=70`

