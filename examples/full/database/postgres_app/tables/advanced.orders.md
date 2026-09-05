# `advanced.orders`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain` |
| `customer_id` | `bigint` | no | - | storage `plain` |
| `region` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |
| `amount` | `numeric(12,2)` | no | - | storage `main` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `orders_amount_not_null` | `not_null` | `amount` | `NOT NULL amount` |
| `orders_customer_id_not_null` | `not_null` | `customer_id` | `NOT NULL customer_id` |
| `orders_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `orders_pkey` | `primary_key` | `id` | `PRIMARY KEY (id)`; no inherit |
| `orders_region_not_null` | `not_null` | `region` | `NOT NULL region` |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `orders_pkey` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `orders_pkey` | - |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

