# `warehouse.analytics.accounts`

User accounts

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `BIGINT` | no | - |  |
| `account_id` | `BIGINT` | no | `nextval('analytics.account_id_seq')` |  |
| `email` | `VARCHAR` | no | - | Canonical email address |
| `normalized_email` | `VARCHAR` | yes | - | generated as `CAST(lower(email) AS VARCHAR)` |
| `status` | `ENUM('active', 'disabled')` | no | `'active'` |  |
| `balance` | `DECIMAL(18,2)` | no | `0` |  |
| `metadata` | `STRUCT("source" VARCHAR, tags VARCHAR[])` | yes | - |  |
| `typed_pair` | `STRUCT(account_id BIGINT, tenant_id BIGINT)` | yes | - |  |
| `typed_reference` | `UNION(account_id BIGINT, external_id VARCHAR)` | yes | - |  |
| `retry_count` | `INTEGER` | yes | - |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `accounts_tenant_id_not_null` | `not null` | `tenant_id` | `NOT NULL` |
| `accounts_tenant_id_tenant_id_fkey` | `foreign key` | `tenant_id` | `FOREIGN KEY (tenant_id) REFERENCES analytics.tenants(tenant_id); references tenants (tenant_id)` |
| `accounts_account_id_pkey` | `primary key` | `account_id` | `PRIMARY KEY(account_id)` |
| `accounts_email_not_null` | `not null` | `email` | `NOT NULL` |
| `accounts_status_not_null` | `not null` | `status` | `NOT NULL` |
| `accounts_balance_not_null` | `not null` | `balance` | `NOT NULL` |
| `accounts_tenant_id_email_key` | `unique` | `tenant_id, email` | `UNIQUE(tenant_id, email)` |
| `accounts_balance_check` | `check` | `balance` | `CHECK((balance >= 0))` |
| `accounts_account_id_not_null` | `not null` | `account_id` | `NOT NULL` |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_email_idx` | `[email]` | no | duckdb; ART | - |


## DuckDB

**Temporary:** no

```sql
CREATE TABLE analytics.accounts(tenant_id BIGINT NOT NULL, account_id BIGINT DEFAULT(nextval('analytics.account_id_seq')) PRIMARY KEY, email VARCHAR NOT NULL, normalized_email VARCHAR GENERATED ALWAYS AS(lower(email)), status ENUM('active', 'disabled') DEFAULT('active') NOT NULL, balance DECIMAL(18,2) DEFAULT(0) NOT NULL, metadata STRUCT("source" VARCHAR, tags VARCHAR[]), typed_pair STRUCT(account_id BIGINT, tenant_id BIGINT), typed_reference UNION(account_id BIGINT, external_id VARCHAR), retry_count INTEGER, FOREIGN KEY (tenant_id) REFERENCES analytics.tenants(tenant_id), UNIQUE(tenant_id, email), CHECK((balance >= 0)));
```
