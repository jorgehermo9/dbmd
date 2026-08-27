# `automation.invoices`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | `nextval('automation.invoice_number'::regclass)` | storage `plain` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `invoices_id_not_null` | `not_null` | `id` | `NOT NULL id` |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

