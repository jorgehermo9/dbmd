# Database: `DuckDB analytics warehouse`

## Schemas

| Name | Comment |
|---|---|
| `warehouse.analytics` | `duckdb` catalog; read-only; catalog tags `storage_version=v1.0.0+` |


## Tables

### `warehouse.analytics.accounts`

User accounts

#### Columns

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


#### Constraints

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


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_email_idx` | `[email]` | no | duckdb; ART | - |


#### DuckDB

**Temporary:** no

```sql
CREATE TABLE analytics.accounts(tenant_id BIGINT NOT NULL, account_id BIGINT DEFAULT(nextval('analytics.account_id_seq')) PRIMARY KEY, email VARCHAR NOT NULL, normalized_email VARCHAR GENERATED ALWAYS AS(lower(email)), status ENUM('active', 'disabled') DEFAULT('active') NOT NULL, balance DECIMAL(18,2) DEFAULT(0) NOT NULL, metadata STRUCT("source" VARCHAR, tags VARCHAR[]), typed_pair STRUCT(account_id BIGINT, tenant_id BIGINT), typed_reference UNION(account_id BIGINT, external_id VARCHAR), retry_count INTEGER, FOREIGN KEY (tenant_id) REFERENCES analytics.tenants(tenant_id), UNIQUE(tenant_id, email), CHECK((balance >= 0)));
```


### `warehouse.analytics.tenants`

Application tenants

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `BIGINT` | no | - |  |
| `name` | `VARCHAR` | no | - |  |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `tenants_tenant_id_pkey` | `primary key` | `tenant_id` | `PRIMARY KEY(tenant_id)` |
| `tenants_name_not_null` | `not null` | `name` | `NOT NULL` |
| `tenants_name_key` | `unique` | `name` | `UNIQUE("name")` |
| `tenants_tenant_id_not_null` | `not null` | `tenant_id` | `NOT NULL` |


#### DuckDB

**Temporary:** no

```sql
CREATE TABLE analytics.tenants(tenant_id BIGINT PRIMARY KEY, "name" VARCHAR NOT NULL UNIQUE);
```


## Views

### `warehouse.analytics.active_accounts`

Active accounts only

**Temporary:** no

| Column | Type | Nullable |
|---|---|---|


```sql
CREATE VIEW analytics.active_accounts AS SELECT tenant_id, account_id, email FROM analytics.accounts WHERE (status = 'active');
```

## Types, Sequences, Functions, and Extensions

### `warehouse.analytics.account_pair`

**Kind:** `type`

**Category:** `COMPOSITE`

**Logical type:** `STRUCT`

**Definition:** `STRUCT(account_id BIGINT, tenant_id BIGINT)`

**Size:** 0



### `warehouse.analytics.account_status`

Account lifecycle

**Kind:** `type`

**Logical type:** `ENUM`

**Definition:** `ENUM('active', 'disabled')`

**Size:** 1

**Labels:** `active, disabled`



### `warehouse.analytics.positive_integer`

**Kind:** `type`

**Category:** `NUMERIC`

**Logical type:** `INTEGER`

**Definition:** `INTEGER`

**Size:** 4



### `warehouse.analytics.reference_value`

**Kind:** `type`

**Category:** `COMPOSITE`

**Logical type:** `UNION`

**Definition:** `UNION(account_id BIGINT, external_id VARCHAR)`

**Size:** 0



### `warehouse.analytics.account_id_seq`

Account identifiers

**Kind:** `sequence`

**Start:** 1000

**Minimum:** 1

**Maximum:** 9223372036854775807

**Increment:** 10

**Cycle:** no

```sql
CREATE SEQUENCE account_id_seq INCREMENT BY 10 MINVALUE 1 MAXVALUE 9223372036854775807 START 1000 NO CYCLE;
```


### `warehouse.analytics.accounts_for_tenant`

**Kind:** `table macro`

**Return type:** `-`

**Parameters:** `owner_id`

**Side effects:** unknown

```sql
SELECT * FROM analytics.accounts WHERE (tenant_id = owner_id)
```


### `warehouse.analytics.normalize_email`

**Kind:** `macro`

**Return type:** `-`

**Parameters:** `value`

**Side effects:** unknown

```sql
lower("value")
```


### `core_functions`

Core function library

**Kind:** `extension`

**Loaded:** yes

**Installed:** yes

**Version:** `-`

**Aliases:** `-`

**Install mode:** `statically linked`

**Installed from:** `-`



