# `catalog.accounts`

Application accounts

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | identity `always`; storage `plain` |
| `email` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |
| `state` | `catalog.account_state` | no | `'active'::catalog.account_state` | enum values `active, suspended`; storage `plain` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `accounts_email_not_null` | `not_null` | `email` | `NOT NULL email` |
| `accounts_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `accounts_pkey` | `primary_key` | `id` | `PRIMARY KEY (id)`; no inherit |
| `accounts_state_not_null` | `not_null` | `state` | `NOT NULL state` |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_pkey` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `accounts_pkey` | - |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

