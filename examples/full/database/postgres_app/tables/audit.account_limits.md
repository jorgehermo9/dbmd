# `audit.account_limits`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `account_id` | `bigint` | no | - | storage `plain` |
| `minimum_balance` | `integer` | no | - | storage `plain` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `account_limits_account_id_not_null` | `not_null` | `account_id` | `NOT NULL account_id` |
| `account_limits_minimum_balance_not_null` | `not_null` | `minimum_balance` | `NOT NULL minimum_balance` |
| `account_limits_pkey` | `primary_key` | `account_id` | `PRIMARY KEY (account_id)`; no inherit |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `account_limits_pkey` | `account_id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `account_limits_pkey` | - |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

