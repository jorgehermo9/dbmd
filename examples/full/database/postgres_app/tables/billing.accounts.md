# `billing.accounts`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `account_id` | `bigint` | no | - | storage `plain` |
| `email` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `accounts_account_id_not_null` | `not_null` | `account_id` | `NOT NULL account_id` |
| `accounts_pk` | `primary_key` | `tenant_id, account_id` | `PRIMARY KEY (tenant_id, account_id)`; no inherit |
| `accounts_tenant_email_unique` | `unique` | `tenant_id, email` | `UNIQUE (tenant_id, email)`; no inherit |
| `accounts_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_pk` | `tenant_id` ascending opclass `pg_catalog.int8_ops` nulls `last`, `account_id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `accounts_pk` | - |
| `accounts_tenant_email_unique` | `tenant_id` ascending opclass `pg_catalog.int8_ops` nulls `last`, `email` ascending collate `pg_catalog."default"` opclass `pg_catalog.text_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `accounts_tenant_email_unique` | - |


## PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

