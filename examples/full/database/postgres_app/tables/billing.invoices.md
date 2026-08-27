# `billing.invoices`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `account_id` | `bigint` | no | - | storage `plain` |
| `invoice_number` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `invoices_account_fk` | `foreign_key` | `tenant_id, account_id` | `FOREIGN KEY (tenant_id, account_id) REFERENCES billing.accounts(tenant_id, account_id) MATCH FULL ON UPDATE CASCADE ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED`; no inherit |
| `invoices_account_id_not_null` | `not_null` | `account_id` | `NOT NULL account_id` |
| `invoices_invoice_number_not_null` | `not_null` | `invoice_number` | `NOT NULL invoice_number` |
| `invoices_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

