# Database Context

## Source: `analytics` — `Analytical warehouse`

Backend: `duckdb`

### Schemas

| Name | Comment |
|---|---|
| `warehouse.analytics` | `duckdb` catalog; read-only; catalog tags `storage_version=v1.0.0+` |


### Tables

#### `warehouse.analytics.accounts`

User accounts

##### Columns

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


##### Constraints

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


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_email_idx` | `[email]` | no | duckdb; ART | - |


##### DuckDB

**Temporary:** no

```sql
CREATE TABLE analytics.accounts(tenant_id BIGINT NOT NULL, account_id BIGINT DEFAULT(nextval('analytics.account_id_seq')) PRIMARY KEY, email VARCHAR NOT NULL, normalized_email VARCHAR GENERATED ALWAYS AS(lower(email)), status ENUM('active', 'disabled') DEFAULT('active') NOT NULL, balance DECIMAL(18,2) DEFAULT(0) NOT NULL, metadata STRUCT("source" VARCHAR, tags VARCHAR[]), typed_pair STRUCT(account_id BIGINT, tenant_id BIGINT), typed_reference UNION(account_id BIGINT, external_id VARCHAR), retry_count INTEGER, FOREIGN KEY (tenant_id) REFERENCES analytics.tenants(tenant_id), UNIQUE(tenant_id, email), CHECK((balance >= 0)));
```


#### `warehouse.analytics.tenants`

Application tenants

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `BIGINT` | no | - |  |
| `name` | `VARCHAR` | no | - |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `tenants_tenant_id_pkey` | `primary key` | `tenant_id` | `PRIMARY KEY(tenant_id)` |
| `tenants_name_not_null` | `not null` | `name` | `NOT NULL` |
| `tenants_name_key` | `unique` | `name` | `UNIQUE("name")` |
| `tenants_tenant_id_not_null` | `not null` | `tenant_id` | `NOT NULL` |


##### DuckDB

**Temporary:** no

```sql
CREATE TABLE analytics.tenants(tenant_id BIGINT PRIMARY KEY, "name" VARCHAR NOT NULL UNIQUE);
```


### Views

#### `warehouse.analytics.active_accounts`

Active accounts only

**Temporary:** no

| Column | Type | Nullable |
|---|---|---|


```sql
CREATE VIEW analytics.active_accounts AS SELECT tenant_id, account_id, email FROM analytics.accounts WHERE (status = 'active');
```

### Types, Sequences, Functions, and Extensions

#### `warehouse.analytics.account_pair`

**Kind:** `type`

**Category:** `COMPOSITE`

**Logical type:** `STRUCT`

**Definition:** `STRUCT(account_id BIGINT, tenant_id BIGINT)`

**Size:** 0



#### `warehouse.analytics.account_status`

Account lifecycle

**Kind:** `type`

**Logical type:** `ENUM`

**Definition:** `ENUM('active', 'disabled')`

**Size:** 1

**Labels:** `active, disabled`



#### `warehouse.analytics.positive_integer`

**Kind:** `type`

**Category:** `NUMERIC`

**Logical type:** `INTEGER`

**Definition:** `INTEGER`

**Size:** 4



#### `warehouse.analytics.reference_value`

**Kind:** `type`

**Category:** `COMPOSITE`

**Logical type:** `UNION`

**Definition:** `UNION(account_id BIGINT, external_id VARCHAR)`

**Size:** 0



#### `warehouse.analytics.account_id_seq`

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


#### `warehouse.analytics.accounts_for_tenant`

**Kind:** `table macro`

**Return type:** `-`

**Parameters:** `owner_id`

**Side effects:** unknown

```sql
SELECT * FROM analytics.accounts WHERE (tenant_id = owner_id)
```


#### `warehouse.analytics.normalize_email`

**Kind:** `macro`

**Return type:** `-`

**Parameters:** `value`

**Side effects:** unknown

```sql
lower("value")
```


#### `core_functions`

Core function library

**Kind:** `extension`

**Loaded:** yes

**Installed:** yes

**Version:** `-`

**Aliases:** `-`

**Install mode:** `statically linked`

**Installed from:** `-`



## Source: `app` — `Transactional application`

Backend: `sqlite`

### Namespaces

| Name | Comment |
|---|---|
| `main` | - |


### Tables

#### `main.accounts`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `account_id` | `INTEGER` | no | - |  |
| `organization_id` | `INTEGER` | no | - |  |
| `email` | `TEXT` | no | - | collate `NOCASE` |
| `normalized_email` | `TEXT` | yes | - | stored_generated; as `lower (email)` |
| `status` | `TEXT` | no | `'active'` |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `account_id` | - |
| - | `not_null` | `organization_id` | - |
| - | `not_null` | `email` | - |
| - | `not_null` | `status` | - |
| - | `check` | `status` | `status IN ('active', 'disabled')` |
| - | `foreign_key` | `organization_id` | references `main.organizations(organization_id)`; update `cascade`, delete `restrict`; not deferrable |
| - | `unique` | `organization_id, email` | - |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_active_email_idx` | `normalized_email` ascending collate `BINARY` | no | `create_index` | `status = 'active'` |
| `sqlite_autoindex_accounts_1` | `organization_id` ascending collate `BINARY`, `email` ascending collate `NOCASE` | yes | `unique_constraint` | - |


##### SQLite

**Kind:** `ordinary`

Strict table.

```sql
CREATE TABLE accounts (
    account_id INTEGER PRIMARY KEY,
    organization_id INTEGER NOT NULL,
    email TEXT NOT NULL COLLATE NOCASE,
    normalized_email TEXT GENERATED ALWAYS AS (lower(email)) STORED,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    FOREIGN KEY (organization_id) REFERENCES organizations (organization_id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    UNIQUE (organization_id, email)
) STRICT
```


#### `main.organizations`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `organization_id` | `INTEGER` | no | - |  |
| `slug` | `TEXT` | no | - |  |
| `name` | `TEXT` | no | - |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `organization_id` | - |
| - | `not_null` | `slug` | - |
| - | `unique` | `slug` | - |
| - | `not_null` | `name` | - |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `sqlite_autoindex_organizations_1` | `slug` ascending collate `BINARY` | yes | `unique_constraint` | - |


##### SQLite

**Kind:** `ordinary`

Strict table.

```sql
CREATE TABLE organizations (
    organization_id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
) STRICT
```


### Views

#### `main.active_accounts`

| Column | Type | Nullable |
|---|---|---|
| `account_id` | `INTEGER` | unknown |
| `organization_id` | `INTEGER` | unknown |
| `email` | `TEXT` | unknown |


```sql
CREATE VIEW active_accounts AS
SELECT account_id, organization_id, email
FROM accounts
WHERE status = 'active'
```

