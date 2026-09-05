# `temporal.accounts`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `account_id` | `bigint` | yes | - | storage `plain` |
| `email` | `text` | no | - | collation `temporal.unicode_fast`; storage `extended` |
| `base_amount` | `integer` | yes | - | storage `plain` |
| `virtual_amount` | `integer` | yes | - | generated `virtual` as `base_amount * 2`; storage `plain` |
| `stored_amount` | `integer` | yes | - | generated `stored` as `base_amount * 3`; storage `plain` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `accounts_amount_nonnegative` | `check` | `base_amount` | `CHECK (base_amount >= 0) NOT ENFORCED`; not validated; not enforced |
| `accounts_email_required` | `not_null` | `email` | `NOT NULL email` |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

