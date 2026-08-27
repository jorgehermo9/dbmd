# `advanced.deleted_orders`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | yes | - | storage `plain` |
| `deleted_at` | `timestamp with time zone` | no | `CURRENT_TIMESTAMP` | storage `plain` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `deleted_orders_deleted_at_not_null` | `not_null` | `deleted_at` | `NOT NULL deleted_at` |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

