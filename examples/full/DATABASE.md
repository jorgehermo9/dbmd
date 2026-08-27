# Database Context

## Source: `sqlite_app` — `SQLite application`

Backend: `sqlite`

### Namespaces

| Name | Comment |
|---|---|
| `main` | - |


### Tables

#### `main.account_search`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `email` | `` | yes | - |  |
| `account_search` | `` | yes | - | virtual_table_hidden |
| `rank` | `` | yes | - | virtual_table_hidden |


##### SQLite

**Kind:** `virtual` using `fts5` with arguments `email, content='accounts', content_rowid='id'`

```sql
CREATE VIRTUAL TABLE account_search USING fts5(email, content='accounts', content_rowid='id')
```


#### `main.account_search_config`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `k` | `` | no | - |  |
| `v` | `` | yes | - |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `k` | - |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `sqlite_autoindex_account_search_config_1` | `k` ascending collate `BINARY` | yes | `primary_key` | - |


##### SQLite

**Kind:** `shadow` owned by `account_search`

Without rowid.

```sql
CREATE TABLE 'account_search_config'(k PRIMARY KEY, v) WITHOUT ROWID
```


#### `main.account_search_data`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `INTEGER` | no | - |  |
| `block` | `BLOB` | yes | - |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `id` | - |


##### SQLite

**Kind:** `shadow` owned by `account_search`

```sql
CREATE TABLE 'account_search_data'(id INTEGER PRIMARY KEY, block BLOB)
```


#### `main.account_search_docsize`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `INTEGER` | no | - |  |
| `sz` | `BLOB` | yes | - |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `id` | - |


##### SQLite

**Kind:** `shadow` owned by `account_search`

```sql
CREATE TABLE 'account_search_docsize'(id INTEGER PRIMARY KEY, sz BLOB)
```


#### `main.account_search_idx`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `segid` | `` | no | - |  |
| `term` | `` | no | - |  |
| `pgno` | `` | yes | - |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `segid, term` | - |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `sqlite_autoindex_account_search_idx_1` | `segid` ascending collate `BINARY`, `term` ascending collate `BINARY` | yes | `primary_key` | - |


##### SQLite

**Kind:** `shadow` owned by `account_search`

Without rowid.

```sql
CREATE TABLE 'account_search_idx'(segid, term, pgno, PRIMARY KEY(segid, term)) WITHOUT ROWID
```


#### `main.accounts`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `INTEGER` | no | - |  |
| `tenant_id` | `INTEGER` | no | - |  |
| `organization_slug` | `TEXT` | no | - |  |
| `email` | `TEXT` | yes | - | collate `NOCASE` |
| `balance_cents` | `INTEGER` | no | `0` |  |
| `normalized_email` | `TEXT` | yes | - | stored_generated; as `lower (email)` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `accounts_pk` | `primary_key` | `id` | -; autoincrement |
| - | `not_null` | `tenant_id` | - |
| - | `not_null` | `organization_slug` | - |
| `accounts_email_key` | `unique` | `email` | -; conflict `ignore` |
| - | `not_null` | `balance_cents` | - |
| `accounts_balance_check` | `check` | `balance_cents` | `balance_cents >= 0` |
| `accounts_organization_fk` | `foreign_key` | `tenant_id, organization_slug` | references `main.organizations(tenant_id, slug)`; update `cascade`, delete `restrict`; match `simple`; deferrable initially `deferred` |
| `accounts_email_check` | `check` | `` | `length (email) > 3` |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_active_email_idx` | `tenant_id` ascending collate `BINARY`, `lower (email)` descending collate `NOCASE` | yes | `create_index` | `balance_cents >= 0` |
| `sqlite_autoindex_accounts_1` | `email` ascending collate `NOCASE` | yes | `unique_constraint` | - |


##### SQLite

**Kind:** `ordinary`

Strict table.

```sql
CREATE TABLE accounts (
    id INTEGER CONSTRAINT accounts_pk PRIMARY KEY AUTOINCREMENT,
    tenant_id INTEGER NOT NULL,
    organization_slug TEXT NOT NULL,
    email TEXT COLLATE NOCASE CONSTRAINT accounts_email_key UNIQUE ON CONFLICT IGNORE,
    balance_cents INTEGER NOT NULL DEFAULT (0)
        CONSTRAINT accounts_balance_check CHECK (balance_cents >= 0),
    normalized_email TEXT GENERATED ALWAYS AS (lower(email)) STORED,
    CONSTRAINT accounts_organization_fk
        FOREIGN KEY (tenant_id, organization_slug)
        REFERENCES organizations
        MATCH simple
        ON UPDATE CASCADE
        ON DELETE RESTRICT
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT accounts_email_check CHECK (length(email) > 3)
) STRICT
```


#### `main.imported`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `` | yes | - |  |
| `label` | `` | yes | - |  |


##### SQLite

**Kind:** `ordinary`

```sql
CREATE TABLE imported(id,label)
```


#### `main.migration_target`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `INTEGER` | no | - |  |
| `name` | `TEXT` | no | - |  |
| `generated_name` | `TEXT` | yes | - | virtual_generated; as `upper (name)` |
| `optional_note` | `TEXT` | yes | - |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `id` | - |
| - | `not_null` | `name` | - |


##### SQLite

**Kind:** `ordinary`

```sql
CREATE TABLE "migration_target" (id INTEGER PRIMARY KEY, name TEXT NOT NULL, generated_name TEXT AS (upper(name)) VIRTUAL, optional_note TEXT)
```


#### `main.organizations`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `INTEGER` | no | - |  |
| `slug` | `TEXT` | no | - | collate `NOCASE` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `not_null` | `tenant_id` | - |
| - | `not_null` | `slug` | - |
| `organizations_pk` | `primary_key` | `tenant_id, slug` | - |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `sqlite_autoindex_organizations_1` | `tenant_id` ascending collate `BINARY`, `slug` ascending collate `NOCASE` | yes | `primary_key` | - |


##### SQLite

**Kind:** `ordinary`

Strict table.

Without rowid.

```sql
CREATE TABLE organizations (
    tenant_id INTEGER NOT NULL,
    slug TEXT COLLATE NOCASE NOT NULL,
    CONSTRAINT organizations_pk PRIMARY KEY (tenant_id, slug)
) WITHOUT ROWID, STRICT
```


### Views

#### `main.account_balances`

| Column | Type | Nullable |
|---|---|---|
| `id` | `INTEGER` | unknown |
| `balance_cents` | `INTEGER` | unknown |


```sql
CREATE VIEW account_balances AS
SELECT id, balance_cents FROM accounts
```

#### `main.account_directory`

| Column | Type | Nullable |
|---|---|---|
| `account_id` | `INTEGER` | unknown |
| `email` | `TEXT` | unknown |


```sql
CREATE VIEW account_directory (account_id, email) AS
SELECT id, email FROM accounts
```

### Triggers

#### `main.account_directory.account_directory_insert`

**INSTEAD OF INSERT** on `main.account_directory`.

```sql
CREATE TRIGGER account_directory_insert
INSTEAD OF INSERT ON account_directory
BEGIN
    INSERT INTO accounts (id, tenant_id, organization_slug, email)
    VALUES (NEW.account_id, 1, 'default', NEW.email);
END
```

#### `main.accounts.accounts_normalize_email`

**AFTER UPDATE OF email** on `main.accounts`.

When: `NEW.email <> lower (NEW.email)`

```sql
CREATE TRIGGER accounts_normalize_email
AFTER UPDATE OF email ON accounts
WHEN NEW.email != lower(NEW.email)
BEGIN
    UPDATE accounts SET email = lower(NEW.email) WHERE id = NEW.id;
END
```

#### `main.accounts.accounts_prevent_root_delete`

**BEFORE DELETE** on `main.accounts`.

When: `OLD.id = 0`

```sql
CREATE TRIGGER accounts_prevent_root_delete
BEFORE DELETE ON accounts
WHEN OLD.id = 0
BEGIN
    SELECT RAISE(IGNORE);
END
```

## Source: `postgres_app` — `PostgreSQL application`

Backend: `postgres`

### Database

#### `dbmd`

**Owner:** `dbmd`

**Encoding:** `UTF8`

**Locale provider:** `libc`

**LC_COLLATE:** `en_US.utf8`

**LC_CTYPE:** `en_US.utf8`

**Tablespace:** `pg_default`

**Template:** no

**Allows connections:** yes

**Connection limit:** -1



### Namespaces

| Name | Owner | Comment |
|---|---|---|
| `advanced` | `dbmd` | - |
| `aggregates` | `dbmd` | - |
| `audit` | `dbmd` | - |
| `automation` | `dbmd` | - |
| `billing` | `dbmd` | - |
| `catalog` | `dbmd` | - |
| `infrastructure` | `dbmd` | - |
| `public` | `pg_database_owner` | standard public schema |
| `routines` | `dbmd` | - |
| `search` | `dbmd` | - |
| `secure` | `dbmd_acl_owner` | - |
| `storage` | `dbmd` | - |
| `temporal` | `dbmd` | - |
| `tenancy` | `dbmd` | - |
| `type_system` | `dbmd` | - |


### Enum Types

| Type | Owner | Values | Comment |
|---|---|---|---|
| `catalog.account_state` | `dbmd` | `active, suspended` | - |
| `infrastructure.label_a` | `dbmd` | `a, b` | - |
| `infrastructure.label_b` | `dbmd` | `a, b` | - |
| `secure.event_state` | `dbmd_acl_owner` | `pending, complete` | - |


### Composite Types

#### `storage.device_row`

**Owner:** `dbmd`

**Attribute:** `device_id` `bigint`

**Attribute:** `payload` `text`; collation `pg_catalog."default"`

```sql
CREATE TYPE "storage"."device_row" AS (
    "device_id" bigint,
    "payload" text COLLATE pg_catalog."default"
);
```


### Domains

#### `secure.event_code`

**Base type:** `text`

**Nullable:** yes

**Owner:** `dbmd_acl_owner`

**Collation:** `pg_catalog."default"`

**Constraint:** `event_code_check`: `CHECK (VALUE <> ''::text)`

```sql
CREATE DOMAIN "secure"."event_code"
    AS text
    COLLATE pg_catalog."default"
    CONSTRAINT "event_code_check" CHECK (VALUE <> ''::text);
```


### Base and Shell Types

#### `infrastructure.label_c`

**Kind:** `base`

**Owner:** `dbmd`

**Input:** `infrastructure.label_c_in`

**Output:** `infrastructure.label_c_out`

**Internal length:** 4

**Passed by value:** yes

**Category:** `U`

**Preferred:** no

**Delimiter:** `,`

**Alignment:** `int4`

**Storage:** `plain`

**Collatable:** no

**Array type:** `infrastructure._label_c`

```sql
CREATE TYPE "infrastructure"."label_c" (
    INPUT = infrastructure.label_c_in,
    OUTPUT = infrastructure.label_c_out,
    INTERNALLENGTH = 4,
    PASSEDBYVALUE,
    ALIGNMENT = int4,
    STORAGE = plain,
    CATEGORY = 'U',
    ARRAY_TYPE = infrastructure._label_c
);
```


#### `type_system.pending_value`

Forward-declared shell type

**Kind:** `shell`

**Owner:** `dbmd`

```sql
CREATE TYPE "type_system"."pending_value";
```


#### `type_system.scalar_token`

Integer-backed application token

**Kind:** `base`

**Owner:** `dbmd`

**Input:** `type_system.scalar_token_in`

**Output:** `type_system.scalar_token_out`

**Internal length:** 4

**Passed by value:** yes

**Category:** `N`

**Preferred:** yes

**Delimiter:** `,`

**Alignment:** `int4`

**Storage:** `plain`

**Collatable:** no

**Default:** `0`

**Array type:** `type_system._scalar_token`

```sql
CREATE TYPE "type_system"."scalar_token" (
    INPUT = type_system.scalar_token_in,
    OUTPUT = type_system.scalar_token_out,
    INTERNALLENGTH = 4,
    PASSEDBYVALUE,
    ALIGNMENT = int4,
    STORAGE = plain,
    CATEGORY = 'N',
    PREFERRED = true,
    DEFAULT = '0',
    ARRAY_TYPE = type_system._scalar_token
);
```


### Range and Multirange Types

#### `type_system.measurement_range`

One continuous measurement interval

**Kind:** `range`

**Owner:** `dbmd`

**Subtype:** `double precision`

**Subtype operator class:** `pg_catalog.float8_ops`

**Multirange:** `type_system.measurement_ranges`

**Multirange owner:** `dbmd`

**Subtype difference:** `pg_catalog.float8mi`

**Multirange comment:** Disjoint measurement intervals

```sql
CREATE TYPE "type_system"."measurement_range" AS RANGE (
    SUBTYPE = double precision,
    SUBTYPE_OPCLASS = pg_catalog.float8_ops,
    SUBTYPE_DIFF = pg_catalog.float8mi,
    MULTIRANGE_TYPE_NAME = "type_system"."measurement_ranges"
);
```


### Sequences

#### `audit.accounts_id_seq`

**Owner:** `dbmd`

**Type:** `bigint`

**Start:** 1

**Minimum:** 1

**Maximum:** 9223372036854775807

**Increment:** 1

**Cache:** 1

**Cycle:** no

**Persistence:** `permanent`

**Owned by:** `audit.accounts.id`

```sql
CREATE SEQUENCE "audit"."accounts_id_seq" AS bigint INCREMENT BY 1 MINVALUE 1 MAXVALUE 9223372036854775807 START WITH 1 CACHE 1 NO CYCLE OWNED BY audit.accounts.id;
```


#### `automation.invoice_number`

Stable invoice number allocator

**Owner:** `dbmd`

**Type:** `bigint`

**Start:** 1000

**Minimum:** 1000

**Maximum:** 999999

**Increment:** 5

**Cache:** 20

**Cycle:** yes

**Persistence:** `unlogged`

**Owned by:** `automation.invoices.id`

```sql
CREATE UNLOGGED SEQUENCE "automation"."invoice_number" AS bigint INCREMENT BY 5 MINVALUE 1000 MAXVALUE 999999 START WITH 1000 CACHE 20 CYCLE OWNED BY automation.invoices.id;
```


#### `catalog.accounts_id_seq`

**Owner:** `dbmd`

**Type:** `bigint`

**Start:** 1

**Minimum:** 1

**Maximum:** 9223372036854775807

**Increment:** 1

**Cache:** 1

**Cycle:** no

**Persistence:** `permanent`

**Owned by:** `catalog.accounts.id`

```sql
CREATE SEQUENCE "catalog"."accounts_id_seq" AS bigint INCREMENT BY 1 MINVALUE 1 MAXVALUE 9223372036854775807 START WITH 1 CACHE 1 NO CYCLE OWNED BY catalog.accounts.id;
```


#### `secure.event_sequence`

**Owner:** `dbmd_acl_owner`

**Type:** `bigint`

**Start:** 1

**Minimum:** 1

**Maximum:** 9223372036854775807

**Increment:** 1

**Cache:** 1

**Cycle:** no

**Persistence:** `permanent`

```sql
CREATE SEQUENCE "secure"."event_sequence" AS bigint INCREMENT BY 1 MINVALUE 1 MAXVALUE 9223372036854775807 START WITH 1 CACHE 1 NO CYCLE OWNED BY NONE;
```


### Tables

#### `advanced.deleted_orders`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | yes | - | storage `plain` |
| `deleted_at` | `timestamp with time zone` | no | `CURRENT_TIMESTAMP` | storage `plain` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `deleted_orders_deleted_at_not_null` | `not_null` | `deleted_at` | `NOT NULL deleted_at` |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



#### `advanced.orders`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain` |
| `customer_id` | `bigint` | no | - | storage `plain` |
| `region` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |
| `amount` | `numeric(12,2)` | no | - | storage `main` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `orders_amount_not_null` | `not_null` | `amount` | `NOT NULL amount` |
| `orders_customer_id_not_null` | `not_null` | `customer_id` | `NOT NULL customer_id` |
| `orders_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `orders_pkey` | `primary_key` | `id` | `PRIMARY KEY (id)`; no inherit |
| `orders_region_not_null` | `not_null` | `region` | `NOT NULL region` |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `orders_pkey` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `orders_pkey` | - |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



#### `audit.account_limits`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `account_id` | `bigint` | no | - | storage `plain` |
| `minimum_balance` | `integer` | no | - | storage `plain` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `account_limits_account_id_not_null` | `not_null` | `account_id` | `NOT NULL account_id` |
| `account_limits_minimum_balance_not_null` | `not_null` | `minimum_balance` | `NOT NULL minimum_balance` |
| `account_limits_pkey` | `primary_key` | `account_id` | `PRIMARY KEY (account_id)`; no inherit |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `account_limits_pkey` | `account_id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `account_limits_pkey` | - |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



#### `audit.accounts`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | identity `always`; storage `plain` |
| `email` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |
| `balance` | `integer` | no | `0` | storage `plain` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `accounts_balance_not_null` | `not_null` | `balance` | `NOT NULL balance` |
| `accounts_email_not_null` | `not_null` | `email` | `NOT NULL email` |
| `accounts_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `accounts_pkey` | `primary_key` | `id` | `PRIMARY KEY (id)`; no inherit |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_pkey` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `accounts_pkey` | - |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



#### `audit.partitioned_events`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain` |
| `occurred_on` | `date` | no | - | storage `plain` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `partitioned_events_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `partitioned_events_occurred_on_not_null` | `not_null` | `occurred_on` | `NOT NULL occurred_on` |


##### PostgreSQL

**Kind:** `partitioned_table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Partition key:** `RANGE (occurred_on)`



#### `audit.partitioned_events_2026`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain`; inherited only |
| `occurred_on` | `date` | no | - | storage `plain`; inherited only |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `partitioned_events_id_not_null` | `not_null` | `id` | `NOT NULL id`; inherited |
| `partitioned_events_occurred_on_not_null` | `not_null` | `occurred_on` | `NOT NULL occurred_on`; inherited |


##### PostgreSQL

**Kind:** `partition`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Inherits:** `audit.partitioned_events`

**Partition parent:** `audit.partitioned_events`

**Partition bound:** `FOR VALUES FROM ('2026-01-01') TO ('2027-01-01')`



#### `automation.invoices`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | `nextval('automation.invoice_number'::regclass)` | storage `plain` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `invoices_id_not_null` | `not_null` | `id` | `NOT NULL id` |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



#### `billing.accounts`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `account_id` | `bigint` | no | - | storage `plain` |
| `email` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `accounts_account_id_not_null` | `not_null` | `account_id` | `NOT NULL account_id` |
| `accounts_pk` | `primary_key` | `tenant_id, account_id` | `PRIMARY KEY (tenant_id, account_id)`; no inherit |
| `accounts_tenant_email_unique` | `unique` | `tenant_id, email` | `UNIQUE (tenant_id, email)`; no inherit |
| `accounts_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_pk` | `tenant_id` ascending opclass `pg_catalog.int8_ops` nulls `last`, `account_id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `accounts_pk` | - |
| `accounts_tenant_email_unique` | `tenant_id` ascending opclass `pg_catalog.int8_ops` nulls `last`, `email` ascending collate `pg_catalog."default"` opclass `pg_catalog.text_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `accounts_tenant_email_unique` | - |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



#### `billing.invoices`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `account_id` | `bigint` | no | - | storage `plain` |
| `invoice_number` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `invoices_account_fk` | `foreign_key` | `tenant_id, account_id` | `FOREIGN KEY (tenant_id, account_id) REFERENCES billing.accounts(tenant_id, account_id) MATCH FULL ON UPDATE CASCADE ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED`; no inherit |
| `invoices_account_id_not_null` | `not_null` | `account_id` | `NOT NULL account_id` |
| `invoices_invoice_number_not_null` | `not_null` | `invoice_number` | `NOT NULL invoice_number` |
| `invoices_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



#### `catalog.accounts`

Application accounts

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | identity `always`; storage `plain` |
| `email` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |
| `state` | `catalog.account_state` | no | `'active'::catalog.account_state` | enum values `active, suspended`; storage `plain` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `accounts_email_not_null` | `not_null` | `email` | `NOT NULL email` |
| `accounts_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `accounts_pkey` | `primary_key` | `id` | `PRIMARY KEY (id)`; no inherit |
| `accounts_state_not_null` | `not_null` | `state` | `NOT NULL state` |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_pkey` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `accounts_pkey` | - |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



#### `search.documents`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain` |
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `title` | `text` | no | - | collation `pg_catalog."C"`; storage `extended` |
| `body` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |
| `published` | `boolean` | no | `false` | storage `plain` |
| `active_window` | `int4range` | yes | - | storage `extended` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `documents_active_window_exclude` | `exclusion` | `active_window` | `EXCLUDE USING gist (active_window WITH &&)`; no inherit; operators `pg_catalog.&&(pg_catalog.anyrange,pg_catalog.anyrange)` |
| `documents_body_not_null` | `not_null` | `body` | `NOT NULL body` |
| `documents_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `documents_published_not_null` | `not_null` | `published` | `NOT NULL published` |
| `documents_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |
| `documents_title_check` | `check` | `title` | `CHECK (title <> ''::text) NOT VALID`; not validated; Rejects empty titles |
| `documents_title_not_null` | `not_null` | `title` | `NOT NULL title` |
| `documents_title_unique` | `unique` | `tenant_id, title` | `UNIQUE NULLS NOT DISTINCT (tenant_id, title) DEFERRABLE INITIALLY DEFERRED`; no inherit |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `documents_active_window_exclude` | `active_window` ascending opclass `pg_catalog.range_ops` | no | postgres `gist`; owner `dbmd`; constraint `documents_active_window_exclude` | - |
| `documents_brin_idx` | `id` ascending opclass `pg_catalog.int8_bloom_ops` parameters `n_distinct_per_range=32, false_positive_rate=0.05` | no | postgres `brin`; owner `dbmd` | - |
| `documents_cluster_idx` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | no | postgres `btree`; clustered; owner `dbmd` | - |
| `documents_lookup_idx` | `tenant_id` ascending opclass `pg_catalog.int8_ops` nulls `last`, `lower(title)` descending collate `pg_catalog."C"` opclass `pg_catalog.text_ops` nulls `first` | yes | postgres `btree`; include `body`; nulls not distinct; owner `dbmd`; option `fillfactor=75`; Published-document lookup | `published` |
| `documents_replica_idx` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; replica identity; owner `dbmd` | - |
| `documents_title_unique` | `tenant_id` ascending opclass `pg_catalog.int8_ops` nulls `last`, `title` ascending collate `pg_catalog."C"` opclass `pg_catalog.text_ops` nulls `last` | yes | postgres `btree`; nulls not distinct; owner `dbmd`; constraint `documents_title_unique` | - |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `index`

**Access method:** `heap`



#### `secure.events`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `events_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `events_pkey` | `primary_key` | `id` | `PRIMARY KEY (id)`; no inherit |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `events_pkey` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd_acl_owner`; constraint `events_pkey` | - |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd_acl_owner`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



#### `secure.remote_events`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | yes | - | storage `plain` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


##### PostgreSQL

**Kind:** `foreign_table`

**Owner:** `dbmd_acl_owner`

**Persistence:** `permanent`

**Replica identity:** `nothing`

**Foreign server:** `secure_server`

**Foreign-data wrapper:** `postgres_fdw`

**Foreign option:** `table_name=events`



#### `storage.event_payloads`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `event_id` | `bigint` | no | - | storage `plain` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `external`; compression `lz4`; statistics target 777; option `n_distinct=-0.5` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `event_payloads_event_id_not_null` | `not_null` | `event_id` | `NOT NULL event_id` |
| `event_payloads_pkey` | `primary_key` | `event_id` | `PRIMARY KEY (event_id)`; no inherit |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `event_payloads_pkey` | `event_id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `event_payloads_pkey` | - |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `unlogged`

**Replica identity:** `full`

**Access method:** `heap`

**Option:** `fillfactor=70`



#### `storage.remote_events`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `event_id` | `bigint` | yes | - | storage `plain`; foreign option `remote_name=external_id` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


##### PostgreSQL

**Kind:** `foreign_table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `nothing`

**Foreign server:** `fixture_server`

**Foreign-data wrapper:** `fixture_wrapper`

**Foreign option:** `schema_name=remote`

**Foreign option:** `table_name=events`



#### `storage.typed_devices`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `device_id` | `bigint` | yes | - | storage `plain` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Of type:** `storage.device_row`



#### `temporal.accounts`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `account_id` | `bigint` | yes | - | storage `plain` |
| `email` | `text` | no | - | collation `temporal.unicode_fast`; storage `extended` |
| `base_amount` | `integer` | yes | - | storage `plain` |
| `virtual_amount` | `integer` | yes | - | generated `virtual` as `base_amount * 2`; storage `plain` |
| `stored_amount` | `integer` | yes | - | generated `stored` as `base_amount * 3`; storage `plain` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `accounts_amount_nonnegative` | `check` | `base_amount` | `CHECK (base_amount >= 0) NOT ENFORCED`; not validated; not enforced |
| `accounts_email_required` | `not_null` | `email` | `NOT NULL email` |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



#### `temporal.plan_assignments`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `plan_id` | `bigint` | no | - | storage `plain` |
| `valid_at` | `daterange` | no | - | storage `extended` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `assignments_plan_period` | `foreign_key` | `plan_id, valid_at` | `FOREIGN KEY (plan_id, PERIOD valid_at) REFERENCES temporal.plan_versions(plan_id, PERIOD valid_at) NOT ENFORCED`; not validated; not enforced; temporal; no inherit |
| `plan_assignments_plan_id_not_null` | `not_null` | `plan_id` | `NOT NULL plan_id` |
| `plan_assignments_valid_at_not_null` | `not_null` | `valid_at` | `NOT NULL valid_at` |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



#### `temporal.plan_versions`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `plan_id` | `bigint` | no | - | storage `plain` |
| `valid_at` | `daterange` | no | - | storage `extended` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `plan_versions_identity` | `unique` | `plan_id, valid_at` | `UNIQUE (plan_id, valid_at WITHOUT OVERLAPS)`; temporal; no inherit; operators `pg_catalog.=(bigint,bigint), pg_catalog.&&(pg_catalog.anyrange,pg_catalog.anyrange)` |
| `plan_versions_plan_id_not_null` | `not_null` | `plan_id` | `NOT NULL plan_id` |
| `plan_versions_valid_at_not_null` | `not_null` | `valid_at` | `NOT NULL valid_at` |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `plan_versions_identity` | `plan_id` ascending opclass `public.gist_int8_ops`, `valid_at` ascending opclass `pg_catalog.range_ops` | yes | postgres `gist`; owner `dbmd`; constraint `assignments_plan_period` | - |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



#### `tenancy.base_events`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `payload` | `jsonb` | no | - | storage `extended` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `base_events_payload_not_null` | `not_null` | `payload` | `NOT NULL payload` |
| `base_events_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Policy:** `tenant_events` `select` to `PUBLIC` (restrictive); using `tenant_id = current_setting('app.tenant_id'::text)::bigint`; Restricts events to the active tenant

Row-level security enabled.

Row-level security forced for the table owner.



#### `tenancy.events`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `created_at` | `date` | no | - | storage `plain` |
| `payload` | `jsonb` | no | - | storage `extended` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `events_created_at_not_null` | `not_null` | `created_at` | `NOT NULL created_at` |
| `events_payload_not_null` | `not_null` | `payload` | `NOT NULL payload` |
| `events_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `events_created_idx` | `created_at` ascending opclass `pg_catalog.date_ops` nulls `last` | no | postgres `btree`; owner `dbmd`; partitioned | - |


##### PostgreSQL

**Kind:** `partitioned_table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Partition key:** `RANGE (created_at)`



#### `tenancy.events_2025`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain`; inherited only |
| `created_at` | `date` | no | - | storage `plain`; inherited only |
| `payload` | `jsonb` | no | - | storage `extended`; inherited only |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `events_created_at_not_null` | `not_null` | `created_at` | `NOT NULL created_at`; inherited |
| `events_payload_not_null` | `not_null` | `payload` | `NOT NULL payload`; inherited |
| `events_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id`; inherited |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `events_2025_created_idx` | `created_at` ascending opclass `pg_catalog.date_ops` nulls `last` | no | postgres `btree`; owner `dbmd`; option `fillfactor=76`; parent `tenancy.events_created_idx` | - |


##### PostgreSQL

**Kind:** `partition`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Inherits:** `tenancy.events`

**Partition parent:** `tenancy.events`

**Partition bound:** `FOR VALUES FROM ('2025-01-01') TO ('2026-01-01')`



#### `tenancy.special_events`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain`; inherited only |
| `payload` | `jsonb` | no | - | storage `extended`; inherited only |
| `category` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `base_events_payload_not_null` | `not_null` | `payload` | `NOT NULL payload`; inherited |
| `base_events_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id`; inherited |
| `special_events_category_not_null` | `not_null` | `category` | `NOT NULL category` |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Inherits:** `tenancy.base_events`



#### `type_system.measurements`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `token` | `type_system.scalar_token` | no | - | storage `plain` |
| `accepted` | `type_system.measurement_range` | no | - | storage `extended` |
| `historical` | `type_system.measurement_ranges` | no | - | storage `extended` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `measurements_accepted_not_null` | `not_null` | `accepted` | `NOT NULL accepted` |
| `measurements_historical_not_null` | `not_null` | `historical` | `NOT NULL historical` |
| `measurements_token_not_null` | `not_null` | `token` | `NOT NULL token` |


##### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



### Views

#### `audit.account_emails`

Writable account email projection

**Kind:** `view`

**Owner:** `dbmd`

**Persistence:** `permanent`

| Column | Type | Nullable |
|---|---|---|
| `id` | `bigint` | yes |
| `email` | `text` | yes |


```sql
 SELECT id,
    email
   FROM audit.accounts;
```

#### `catalog.active_accounts`

**Kind:** `view`

**Owner:** `dbmd`

**Persistence:** `permanent`

| Column | Type | Nullable |
|---|---|---|
| `id` | `bigint` | yes |
| `email` | `text` | yes |


```sql
 SELECT id,
    email
   FROM catalog.accounts
  WHERE state = 'active'::catalog.account_state;
```

#### `secure.event_rollup`

**Kind:** `materialized_view`

**Owner:** `dbmd_acl_owner`

**Persistence:** `permanent`

**Populated:** no

**Access method:** `heap`

| Column | Type | Nullable |
|---|---|---|
| `event_count` | `bigint` | yes |


```sql
 SELECT count(*) AS event_count
   FROM secure.events;
```

#### `secure.event_view`

**Kind:** `view`

**Owner:** `dbmd_acl_owner`

**Persistence:** `permanent`

| Column | Type | Nullable |
|---|---|---|
| `id` | `bigint` | yes |
| `payload` | `text` | yes |


```sql
 SELECT id,
    payload
   FROM secure.events;
```

### Triggers

#### `audit.account_emails.account_emails_write`

**INSTEAD OF INSERT OR UPDATE OR DELETE** on `audit.account_emails`.

**Orientation:** `row`

**Function:** `audit.capture_row_change()`

**Enabled:** `origin`

**Arguments:** `view`

```sql
CREATE TRIGGER account_emails_write INSTEAD OF INSERT OR DELETE OR UPDATE ON audit.account_emails FOR EACH ROW EXECUTE FUNCTION audit.capture_row_change('view')
```

#### `audit.accounts.accounts_balance_constraint`

**AFTER UPDATE OF balance** on `audit.accounts`.

**Orientation:** `row`

**Function:** `audit.capture_row_change()`

**Enabled:** `origin`

**Arguments:** `balance`

**Constraint trigger:** deferrable initially deferred; from `audit.account_limits`

When: `new.balance < 0`

```sql
CREATE CONSTRAINT TRIGGER accounts_balance_constraint AFTER UPDATE OF balance ON audit.accounts FROM audit.account_limits DEFERRABLE INITIALLY DEFERRED FOR EACH ROW WHEN (new.balance < 0) EXECUTE FUNCTION audit.capture_row_change('balance')
```

#### `audit.accounts.accounts_transition`

**AFTER UPDATE** on `audit.accounts`.

**Orientation:** `statement`

**Function:** `audit.capture_statement_change()`

**Enabled:** `disabled`

**Old transition table:** `previous_rows`

**New transition table:** `current_rows`

```sql
CREATE TRIGGER accounts_transition AFTER UPDATE ON audit.accounts REFERENCING OLD TABLE AS previous_rows NEW TABLE AS current_rows FOR EACH STATEMENT EXECUTE FUNCTION audit.capture_statement_change()
```

#### `audit.accounts.accounts_truncate`

**AFTER TRUNCATE** on `audit.accounts`.

**Orientation:** `statement`

**Function:** `audit.capture_statement_change()`

**Enabled:** `replica`

```sql
CREATE TRIGGER accounts_truncate AFTER TRUNCATE ON audit.accounts FOR EACH STATEMENT EXECUTE FUNCTION audit.capture_statement_change()
```

#### `audit.accounts.zz_accounts_change`

Captures relevant account row changes

**BEFORE INSERT OR UPDATE OF email, balance OR DELETE** on `audit.accounts`.

**Orientation:** `row`

**Function:** `audit.capture_row_change()`

**Enabled:** `always`

**Arguments:** `history, full`

When: `pg_trigger_depth() = 0`

```sql
CREATE TRIGGER zz_accounts_change BEFORE INSERT OR DELETE OR UPDATE OF email, balance ON audit.accounts FOR EACH ROW WHEN (pg_trigger_depth() = 0) EXECUTE FUNCTION audit.capture_row_change('history', 'full')
```

#### `audit.partitioned_events.partitioned_events_change`

**BEFORE INSERT OR UPDATE** on `audit.partitioned_events`.

**Orientation:** `row`

**Function:** `audit.capture_row_change()`

**Enabled:** `origin`

**Arguments:** `partition`

```sql
CREATE TRIGGER partitioned_events_change BEFORE INSERT OR UPDATE ON audit.partitioned_events FOR EACH ROW EXECUTE FUNCTION audit.capture_row_change('partition')
```

#### `audit.partitioned_events_2026.partitioned_events_change`

**BEFORE INSERT OR UPDATE** on `audit.partitioned_events_2026`.

**Orientation:** `row`

**Function:** `audit.capture_row_change()`

**Enabled:** `origin`

**Arguments:** `partition`

**Parent trigger:** `audit.partitioned_events.partitioned_events_change`

```sql
CREATE TRIGGER partitioned_events_change BEFORE INSERT OR UPDATE ON audit.partitioned_events_2026 FOR EACH ROW EXECUTE FUNCTION audit.capture_row_change('partition')
```

### Functions

#### `advanced.capture_schema_change()`

**Kind:** `ordinary`

**Arguments:** ``

**Returns:** `event_trigger`

**Owner:** `dbmd`

**Language:** `plpgsql`

**Volatility:** `volatile`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION advanced.capture_schema_change()
 RETURNS event_trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    NULL;
END;
$function$

```


#### `aggregates.collect_integer(state integer[], value integer)`

**Kind:** `ordinary`

**Arguments:** `state integer[], value integer`

**Returns:** `integer[]`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `immutable`

**Parallel:** `safe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION aggregates.collect_integer(state integer[], value integer)
 RETURNS integer[]
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE
RETURN array_append(state, value)

```


#### `aggregates.hypothetical_position(state integer[], hypothetical integer)`

**Kind:** `ordinary`

**Arguments:** `state integer[], hypothetical integer`

**Returns:** `bigint`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `immutable`

**Parallel:** `safe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION aggregates.hypothetical_position(state integer[], hypothetical integer)
 RETURNS bigint
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE
RETURN (SELECT (1 + count(*)) FROM unnest(hypothetical_position.state) value(value) WHERE (value.value < hypothetical_position.hypothetical))

```


#### `aggregates.pick_integer(state integer[], fraction double precision)`

**Kind:** `ordinary`

**Arguments:** `state integer[], fraction double precision`

**Returns:** `integer`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `immutable`

**Parallel:** `safe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION aggregates.pick_integer(state integer[], fraction double precision)
 RETURNS integer
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE
RETURN (SELECT value.value FROM unnest(pick_integer.state) value(value) ORDER BY value.value OFFSET LEAST((cardinality(pick_integer.state) - 1), GREATEST(0, (floor((pick_integer.fraction * ((cardinality(pick_integer.state) - 1))::double precision)))::integer)) LIMIT 1)

```


#### `aggregates.total_combine(left_state bigint, right_state bigint)`

**Kind:** `ordinary`

**Arguments:** `left_state bigint, right_state bigint`

**Returns:** `bigint`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `immutable`

**Parallel:** `safe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION aggregates.total_combine(left_state bigint, right_state bigint)
 RETURNS bigint
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE
RETURN (COALESCE(left_state, (0)::bigint) + COALESCE(right_state, (0)::bigint))

```


#### `aggregates.total_final(state bigint)`

**Kind:** `ordinary`

**Arguments:** `state bigint`

**Returns:** `bigint`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `immutable`

**Parallel:** `safe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION aggregates.total_final(state bigint)
 RETURNS bigint
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE
RETURN state

```


#### `aggregates.total_inverse(state bigint, value integer)`

**Kind:** `ordinary`

**Arguments:** `state bigint, value integer`

**Returns:** `bigint`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `immutable`

**Parallel:** `safe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION aggregates.total_inverse(state bigint, value integer)
 RETURNS bigint
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE
RETURN (COALESCE(state, (0)::bigint) - COALESCE(value, 0))

```


#### `aggregates.total_step(state bigint, value integer)`

**Kind:** `ordinary`

**Arguments:** `state bigint, value integer`

**Returns:** `bigint`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `immutable`

**Parallel:** `safe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION aggregates.total_step(state bigint, value integer)
 RETURNS bigint
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE
RETURN (COALESCE(state, (0)::bigint) + COALESCE(value, 0))

```


#### `audit.capture_row_change()`

**Kind:** `ordinary`

**Arguments:** ``

**Returns:** `trigger`

**Owner:** `dbmd`

**Language:** `plpgsql`

**Volatility:** `volatile`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION audit.capture_row_change()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    RETURN COALESCE(NEW, OLD);
END;
$function$

```


#### `audit.capture_statement_change()`

**Kind:** `ordinary`

**Arguments:** ``

**Returns:** `trigger`

**Owner:** `dbmd`

**Language:** `plpgsql`

**Volatility:** `volatile`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION audit.capture_statement_change()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    RETURN NULL;
END;
$function$

```


#### `infrastructure.fixture_btree_handler(internal)`

**Kind:** `ordinary`

**Arguments:** `internal`

**Returns:** `index_am_handler`

**Owner:** `dbmd`

**Language:** `internal`

**Volatility:** `volatile`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** strict

**Leakproof:** no

**Returns set:** no

**Cost:** 1

```sql
CREATE OR REPLACE FUNCTION infrastructure.fixture_btree_handler(internal)
 RETURNS index_am_handler
 LANGUAGE internal
 STRICT
AS $function$bthandler$function$

```


#### `infrastructure.label_a_to_b(infrastructure.label_a)`

**Kind:** `ordinary`

**Arguments:** `infrastructure.label_a`

**Returns:** `infrastructure.label_b`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `immutable`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** strict

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION infrastructure.label_a_to_b(infrastructure.label_a)
 RETURNS infrastructure.label_b
 LANGUAGE sql
 IMMUTABLE STRICT
RETURN (($1)::text)::infrastructure.label_b

```


#### `infrastructure.label_c_in(cstring)`

**Kind:** `ordinary`

**Arguments:** `cstring`

**Returns:** `infrastructure.label_c`

**Owner:** `dbmd`

**Language:** `internal`

**Volatility:** `immutable`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** strict

**Leakproof:** no

**Returns set:** no

**Cost:** 1

```sql
CREATE OR REPLACE FUNCTION infrastructure.label_c_in(cstring)
 RETURNS infrastructure.label_c
 LANGUAGE internal
 IMMUTABLE STRICT
AS $function$int4in$function$

```


#### `infrastructure.label_c_out(infrastructure.label_c)`

**Kind:** `ordinary`

**Arguments:** `infrastructure.label_c`

**Returns:** `cstring`

**Owner:** `dbmd`

**Language:** `internal`

**Volatility:** `immutable`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** strict

**Leakproof:** no

**Returns set:** no

**Cost:** 1

```sql
CREATE OR REPLACE FUNCTION infrastructure.label_c_out(infrastructure.label_c)
 RETURNS cstring
 LANGUAGE internal
 IMMUTABLE STRICT
AS $function$int4out$function$

```


#### `infrastructure.nonzero(integer)`

**Kind:** `ordinary`

**Arguments:** `integer`

**Returns:** `boolean`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `immutable`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** strict

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION infrastructure.nonzero(integer)
 RETURNS boolean
 LANGUAGE sql
 IMMUTABLE STRICT
RETURN ($1 <> 0)

```


#### `infrastructure.same_integer(integer, integer)`

**Kind:** `ordinary`

**Arguments:** `integer, integer`

**Returns:** `boolean`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `immutable`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** strict

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION infrastructure.same_integer(integer, integer)
 RETURNS boolean
 LANGUAGE sql
 IMMUTABLE STRICT
RETURN ($1 = $2)

```


#### `routines.range_values(first_value integer, last_value integer)`

**Kind:** `ordinary`

**Arguments:** `first_value integer, last_value integer`

**Returns:** `SETOF integer`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `stable`

**Parallel:** `restricted`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** yes

**Cost:** 7

**Rows:** 25

**Setting:** `search_path=pg_catalog`

```sql
CREATE OR REPLACE FUNCTION routines.range_values(first_value integer, last_value integer)
 RETURNS SETOF integer
 LANGUAGE sql
 STABLE PARALLEL RESTRICTED COST 7 ROWS 25
 SET search_path TO 'pg_catalog'
AS $function$ SELECT generate_series(first_value, last_value) $function$

```


#### `routines.row_number_clone()`

**Kind:** `window`

**Arguments:** ``

**Returns:** `bigint`

**Owner:** `dbmd`

**Language:** `internal`

**Volatility:** `immutable`

**Parallel:** `safe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 1

```sql
CREATE OR REPLACE FUNCTION routines.row_number_clone()
 RETURNS bigint
 LANGUAGE internal
 WINDOW IMMUTABLE PARALLEL SAFE
AS $function$window_row_number$function$

```


#### `routines.starts_with(value text, prefix text)`

Planner-supported strict and leakproof function

**Kind:** `ordinary`

**Arguments:** `value text, prefix text DEFAULT ''::text`

**Returns:** `boolean`

**Owner:** `dbmd`

**Language:** `internal`

**Volatility:** `immutable`

**Parallel:** `safe`

**Security:** `invoker`

**Null input:** strict

**Leakproof:** yes

**Returns set:** no

**Cost:** 3

**Support function:** `pg_catalog.text_starts_with_support(pg_catalog.internal)`

```sql
CREATE OR REPLACE FUNCTION routines.starts_with(value text, prefix text DEFAULT ''::text)
 RETURNS boolean
 LANGUAGE internal
 IMMUTABLE PARALLEL SAFE STRICT LEAKPROOF COST 3 SUPPORT text_starts_with_support
AS $function$text_starts_with$function$

```


#### `secure.event_count()`

**Kind:** `ordinary`

**Arguments:** ``

**Returns:** `bigint`

**Owner:** `dbmd_acl_owner`

**Language:** `sql`

**Volatility:** `volatile`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION secure.event_count()
 RETURNS bigint
 LANGUAGE sql
RETURN (SELECT count(*) AS count FROM secure.events)

```


#### `type_system.scalar_token_in(cstring)`

**Kind:** `ordinary`

**Arguments:** `cstring`

**Returns:** `type_system.scalar_token`

**Owner:** `dbmd`

**Language:** `internal`

**Volatility:** `immutable`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** strict

**Leakproof:** no

**Returns set:** no

**Cost:** 1

```sql
CREATE OR REPLACE FUNCTION type_system.scalar_token_in(cstring)
 RETURNS type_system.scalar_token
 LANGUAGE internal
 IMMUTABLE STRICT
AS $function$int4in$function$

```


#### `type_system.scalar_token_out(type_system.scalar_token)`

**Kind:** `ordinary`

**Arguments:** `type_system.scalar_token`

**Returns:** `cstring`

**Owner:** `dbmd`

**Language:** `internal`

**Volatility:** `immutable`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** strict

**Leakproof:** no

**Returns set:** no

**Cost:** 1

```sql
CREATE OR REPLACE FUNCTION type_system.scalar_token_out(type_system.scalar_token)
 RETURNS cstring
 LANGUAGE internal
 IMMUTABLE STRICT
AS $function$int4out$function$

```


### Procedures

#### `infrastructure.accept_integer(IN value integer)`

**Arguments:** `IN value integer`

**Owner:** `dbmd`

**Language:** `fixture_pl`

**Security:** `invoker`

**Transform:** `integer`

```sql
CREATE OR REPLACE PROCEDURE infrastructure.accept_integer(IN value integer)
 TRANSFORM FOR TYPE integer
 LANGUAGE fixture_pl
AS $procedure$
BEGIN
    NULL;
END;
$procedure$

```


#### `secure.clear_events()`

**Arguments:** ``

**Owner:** `dbmd_acl_owner`

**Language:** `sql`

**Security:** `invoker`

```sql
CREATE OR REPLACE PROCEDURE secure.clear_events()
 LANGUAGE sql
AS $procedure$DELETE FROM secure.events$procedure$

```


### Aggregates

#### `aggregates.hypothetical_position(integer ORDER BY integer)`

Example hypothetical-set aggregate

**Kind:** `hypothetical_set`

**Arguments:** `integer ORDER BY integer`

**Owner:** `dbmd`

**Returns:** `bigint`

**Direct arguments:** 1

**Transition function:** `aggregates.collect_integer(integer[],integer)`

**Transition type:** `integer[]`

**Transition space:** 0

**Final modify:** `read_write`

**Parallel:** `safe`

**Final function:** `aggregates.hypothetical_position(integer[],integer)`

**Initial condition:** `{}`



#### `aggregates.integer_total(integer)`

Adds integers with ordinary, moving, and parallel aggregation support

**Kind:** `normal`

**Arguments:** `integer`

**Owner:** `dbmd`

**Returns:** `bigint`

**Direct arguments:** 0

**Transition function:** `aggregates.total_step(bigint,integer)`

**Transition type:** `bigint`

**Transition space:** 0

**Final modify:** `shareable`

**Parallel:** `safe`

**Final function:** `aggregates.total_final(bigint)`

**Combine function:** `aggregates.total_combine(bigint,bigint)`

**Moving transition function:** `aggregates.total_step(bigint,integer)`

**Moving inverse function:** `aggregates.total_inverse(bigint,integer)`

**Moving final function:** `aggregates.total_final(bigint)`

**Moving transition type:** `bigint`

**Initial condition:** `0`

**Moving initial condition:** `0`

**Moving transition space:** 0

**Moving final modify:** `read_write`



#### `aggregates.percentile_pick(double precision ORDER BY integer)`

Example ordered-set aggregate

**Kind:** `ordered_set`

**Arguments:** `double precision ORDER BY integer`

**Owner:** `dbmd`

**Returns:** `integer`

**Direct arguments:** 1

**Transition function:** `aggregates.collect_integer(integer[],integer)`

**Transition type:** `integer[]`

**Transition space:** 0

**Final modify:** `read_write`

**Parallel:** `safe`

**Final function:** `aggregates.pick_integer(integer[],double precision)`

**Initial condition:** `{}`



#### `secure.total_int(integer)`

**Kind:** `normal`

**Arguments:** `integer`

**Owner:** `dbmd_acl_owner`

**Returns:** `integer`

**Direct arguments:** 0

**Transition function:** `pg_catalog.int4pl(integer,integer)`

**Transition type:** `integer`

**Transition space:** 0

**Final modify:** `read_only`

**Parallel:** `unsafe`

**Initial condition:** `0`



### Casts

#### `infrastructure.label_a AS infrastructure.label_b`

Fixture implicit cast

**Context:** `implicit`

**Method:** `function`

**Function:** `infrastructure.label_a_to_b(infrastructure.label_a)`



#### `infrastructure.label_b AS infrastructure.label_c`

**Context:** `assignment`

**Method:** `input_output`



#### `infrastructure.label_c AS integer`

**Context:** `explicit`

**Method:** `binary`



### Encoding Conversions

#### `infrastructure.utf8_to_latin1`

Fixture default encoding conversion

**Owner:** `dbmd`

**Source encoding:** `UTF8`

**Target encoding:** `LATIN1`

**Function:** `pg_catalog.utf8_to_iso8859_1(integer,integer,pg_catalog.cstring,pg_catalog.internal,integer,boolean)`

**Default:** yes



### Operators

#### `infrastructure.!!(NONE, integer)`

**Owner:** `dbmd`

**Kind:** `prefix`

**Result:** `boolean`

**Function:** `infrastructure.nonzero(integer)`

**Merge join:** no

**Hash join:** no



#### `infrastructure.===(integer, integer)`

Fixture equality operator

**Owner:** `dbmd`

**Kind:** `binary`

**Result:** `boolean`

**Function:** `infrastructure.same_integer(integer,integer)`

**Merge join:** yes

**Hash join:** yes

**Commutator:** `infrastructure.===(integer,integer)`

**Restriction estimator:** `pg_catalog.eqsel(pg_catalog.internal,pg_catalog.oid,pg_catalog.internal,integer)`

**Join estimator:** `pg_catalog.eqjoinsel(pg_catalog.internal,pg_catalog.oid,pg_catalog.internal,smallint,pg_catalog.internal)`



### Operator Families

#### `infrastructure.integer_family`

Fixture integer operator family

**Owner:** `dbmd`

**Access method:** `btree`

**Operator:** strategy 1; `pg_catalog.<(integer,integer)` (`integer`, `integer`) via `btree`

**Operator:** strategy 2; `pg_catalog.<=(integer,integer)` (`integer`, `integer`) via `btree`

**Operator:** strategy 3; `pg_catalog.=(integer,integer)` (`integer`, `integer`) via `btree`

**Operator:** strategy 4; `pg_catalog.>=(integer,integer)` (`integer`, `integer`) via `btree`

**Operator:** strategy 5; `pg_catalog.>(integer,integer)` (`integer`, `integer`) via `btree`

**Support function:** number 1; `pg_catalog.btint4cmp(integer,integer)` (`integer`, `integer`)



### Operator Classes

#### `infrastructure.integer_class`

Fixture integer operator class

**Owner:** `dbmd`

**Access method:** `btree`

**Family:** `infrastructure.integer_family`

**Input type:** `integer`

**Default:** no



### Access Methods

#### `fixture_btree`

Fixture index access method

**Kind:** `index`

**Handler:** `infrastructure.fixture_btree_handler(pg_catalog.internal)`



### Procedural Languages

#### `fixture_pl`

Fixture procedural language

**Owner:** `dbmd`

**Procedural:** yes

**Trusted:** yes

**Handler:** `pg_catalog.plpgsql_call_handler()`

**Inline handler:** `pg_catalog.plpgsql_inline_handler(pg_catalog.internal)`

**Validator:** `pg_catalog.plpgsql_validator(pg_catalog.oid)`



### Transforms

#### `integer FOR fixture_pl`

Fixture integer transform

**Language:** `fixture_pl`

**From SQL:** `pg_catalog.textlike_support(pg_catalog.internal)`

**To SQL:** `pg_catalog.int4recv(pg_catalog.internal)`



### Rewrite Rules

#### `advanced.orders.archive_order_delete`

Archives replicated deletes

**Event:** `delete`

**Instead:** no

**Enabled:** `replica`

```sql
CREATE RULE archive_order_delete AS
    ON DELETE TO advanced.orders DO  INSERT INTO advanced.deleted_orders (id)
  VALUES (old.id);
```


### Event Triggers

#### `capture_schema_change`

Captures selected schema changes

**Owner:** `dbmd`

**Event:** `DDL command end`

**Function:** `advanced.capture_schema_change()`

**Enabled:** `always`

**Tags:** `CREATE TABLE, ALTER TABLE`

```sql
CREATE EVENT TRIGGER "capture_schema_change" ON ddl_command_end WHEN TAG IN ('CREATE TABLE', 'ALTER TABLE') EXECUTE FUNCTION advanced.capture_schema_change();
```


### Extended Statistics

#### `advanced.orders_dependencies`

Cross-column order distribution

**Owner:** `dbmd`

**Kinds:** `ndistinct, dependencies, mcv`

**Statistics target:** 500

**Columns:** `customer_id, region`

```sql
CREATE STATISTICS advanced.orders_dependencies ON customer_id, region FROM advanced.orders
```


#### `advanced.orders_expression`

**Owner:** `dbmd`

**Kinds:** `expressions`

**Statistics target:** -1

**Expression:** `lower(region)`

```sql
CREATE STATISTICS advanced.orders_expression ON lower(region) FROM advanced.orders
```


### Foreign-Data Wrappers

#### `fixture_wrapper`

Fixture foreign-data wrapper

**Owner:** `dbmd`

**Option:** `api_token=<redacted>`

**Option:** `endpoint=catalog.example`



### Foreign Servers

#### `fixture_server`

Fixture foreign server

**Owner:** `dbmd`

**Foreign-data wrapper:** `fixture_wrapper`

**Type:** `catalog`

**Version:** `1.0`

**Option:** `host=catalog.example`

**Option:** `password=<redacted>`



#### `secure_server`

**Owner:** `dbmd_acl_owner`

**Foreign-data wrapper:** `postgres_fdw`

**Option:** `host=127.0.0.1`

**Option:** `dbname=postgres`



### User Mappings

#### `PUBLIC ON fixture_server`

**Option:** `user=catalog_reader`

**Option:** `password=<redacted>`



### Text Search Parsers

#### `advanced.default_parser`

Fixture parser backed by PostgreSQL defaults

**Start function:** `pg_catalog.prsd_start(pg_catalog.internal,integer)`

**Token function:** `pg_catalog.prsd_nexttoken(pg_catalog.internal,pg_catalog.internal,pg_catalog.internal)`

**End function:** `pg_catalog.prsd_end(pg_catalog.internal)`

**Headline function:** `pg_catalog.prsd_headline(pg_catalog.internal,pg_catalog.internal,pg_catalog.tsquery)`

**Token-types function:** `pg_catalog.prsd_lextype(pg_catalog.internal)`



### Text Search Templates

#### `advanced.simple_template`

Fixture simple dictionary template

**Init function:** `pg_catalog.dsimple_init(pg_catalog.internal)`

**Lexize function:** `pg_catalog.dsimple_lexize(pg_catalog.internal,pg_catalog.internal,pg_catalog.internal,pg_catalog.internal)`



### Text Search Dictionaries

#### `advanced.simple_dictionary`

Fixture stop-word dictionary

**Owner:** `dbmd`

**Template:** `advanced.simple_template`

**Options:** `stopwords = 'english'`



### Text Search Configurations

#### `advanced.search_configuration`

Fixture search pipeline

**Owner:** `dbmd`

**Parser:** `advanced.default_parser`

**Mapping:** `asciiword`: `advanced.simple_dictionary, pg_catalog.english_stem`



### Publications

#### `advanced_publication`

**Owner:** `dbmd`

**All tables:** no

**Actions:** `insert, update, delete, truncate`

**Generated columns:** `none`

**Publish via partition root:** no

**Table:** `advanced.orders`



#### `all_tables`

**Owner:** `dbmd`

**All tables:** yes

**Actions:** `insert`

**Generated columns:** `none`

**Publish via partition root:** no



#### `temporal_changes`

Stored generated values for analytics

**Owner:** `dbmd`

**All tables:** no

**Actions:** `insert`

**Generated columns:** `stored`

**Publish via partition root:** no

**Table:** `temporal.accounts`; columns `account_id, stored_amount`; where `base_amount >= 0`



#### `temporal_schema`

**Owner:** `dbmd`

**All tables:** no

**Actions:** `insert, truncate`

**Generated columns:** `none`

**Publish via partition root:** no

**Schema:** `temporal`



### Subscriptions

#### `advanced_subscription`

Disconnected fixture subscription

**Owner:** `dbmd`

**Enabled:** no

**Binary:** yes

**Streaming:** `parallel`

**Two phase:** `pending`

**Disable on error:** yes

**Password required:** no

**Run as owner:** yes

**Failover:** yes

**Synchronous commit:** `remote apply`

**Publications:** `advanced_publication`

**Origin:** `no origin`

**Connection:** `<redacted>`

**Skip LSN:** `0/16B6C50`



### Object Privileges

#### `aggregate secure.total_int(integer) → PUBLIC EXECUTE`

**Object type:** `aggregate`

**Object:** `secure.total_int(integer)`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `PUBLIC`

**Privilege:** `EXECUTE`



#### `aggregate secure.total_int(integer) → dbmd_acl_owner EXECUTE`

**Object type:** `aggregate`

**Object:** `secure.total_int(integer)`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `EXECUTE`



#### `aggregate secure.total_int(integer) → dbmd_acl_reader EXECUTE`

**Object type:** `aggregate`

**Object:** `secure.total_int(integer)`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `EXECUTE`



#### `database dbmd → PUBLIC CONNECT`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `PUBLIC`

**Privilege:** `CONNECT`



#### `database dbmd → PUBLIC TEMPORARY`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `PUBLIC`

**Privilege:** `TEMPORARY`



#### `database dbmd → dbmd CONNECT`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `CONNECT`



#### `database dbmd → dbmd CREATE`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `CREATE`



#### `database dbmd → dbmd TEMPORARY`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `TEMPORARY`



#### `database dbmd → dbmd_acl_reader CONNECT`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `CONNECT`



#### `database dbmd → dbmd_acl_reader CREATE`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `CREATE`



#### `database dbmd → dbmd_acl_reader TEMPORARY`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `TEMPORARY`



#### `foreign table secure.remote_events → dbmd_acl_owner DELETE`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `DELETE`



#### `foreign table secure.remote_events → dbmd_acl_owner INSERT`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `INSERT`



#### `foreign table secure.remote_events → dbmd_acl_owner MAINTAIN`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `MAINTAIN`



#### `foreign table secure.remote_events → dbmd_acl_owner REFERENCES`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `REFERENCES`



#### `foreign table secure.remote_events → dbmd_acl_owner SELECT`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



#### `foreign table secure.remote_events → dbmd_acl_owner TRIGGER`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRIGGER`



#### `foreign table secure.remote_events → dbmd_acl_owner TRUNCATE`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRUNCATE`



#### `foreign table secure.remote_events → dbmd_acl_owner UPDATE`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



#### `foreign table secure.remote_events → dbmd_acl_reader SELECT`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



#### `foreign-data wrapper postgres_fdw → dbmd USAGE`

**Object type:** `foreign-data wrapper`

**Object:** `postgres_fdw`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `USAGE`



#### `foreign-data wrapper postgres_fdw → dbmd_acl_reader USAGE`

**Object type:** `foreign-data wrapper`

**Object:** `postgres_fdw`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



#### `function secure.event_count() → PUBLIC EXECUTE`

**Object type:** `function`

**Object:** `secure.event_count()`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `PUBLIC`

**Privilege:** `EXECUTE`



#### `function secure.event_count() → dbmd_acl_owner EXECUTE`

**Object type:** `function`

**Object:** `secure.event_count()`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `EXECUTE`



#### `function secure.event_count() → dbmd_acl_reader EXECUTE`

**Object type:** `function`

**Object:** `secure.event_count()`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `EXECUTE`



#### `language plpgsql → PUBLIC USAGE`

**Object type:** `language`

**Object:** `plpgsql`

**Grantor:** `dbmd`

**Grantee:** `PUBLIC`

**Privilege:** `USAGE`



#### `language plpgsql → dbmd USAGE`

**Object type:** `language`

**Object:** `plpgsql`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `USAGE`



#### `language plpgsql → dbmd_acl_reader USAGE`

**Object type:** `language`

**Object:** `plpgsql`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



#### `large object 424242 → dbmd_acl_owner SELECT`

**Object type:** `large object`

**Object:** `424242`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



#### `large object 424242 → dbmd_acl_owner UPDATE`

**Object type:** `large object`

**Object:** `424242`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



#### `large object 424242 → dbmd_acl_reader SELECT`

**Object type:** `large object`

**Object:** `424242`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



#### `large object 424242 → dbmd_acl_reader UPDATE`

**Object type:** `large object`

**Object:** `424242`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `UPDATE`



#### `materialized view secure.event_rollup → dbmd_acl_owner DELETE`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `DELETE`



#### `materialized view secure.event_rollup → dbmd_acl_owner INSERT`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `INSERT`



#### `materialized view secure.event_rollup → dbmd_acl_owner MAINTAIN`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `MAINTAIN`



#### `materialized view secure.event_rollup → dbmd_acl_owner REFERENCES`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `REFERENCES`



#### `materialized view secure.event_rollup → dbmd_acl_owner SELECT`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



#### `materialized view secure.event_rollup → dbmd_acl_owner TRIGGER`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRIGGER`



#### `materialized view secure.event_rollup → dbmd_acl_owner TRUNCATE`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRUNCATE`



#### `materialized view secure.event_rollup → dbmd_acl_owner UPDATE`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



#### `materialized view secure.event_rollup → dbmd_acl_reader SELECT`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



#### `parameter statement_timeout → dbmd ALTER SYSTEM`

**Object type:** `parameter`

**Object:** `statement_timeout`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `ALTER SYSTEM`



#### `parameter statement_timeout → dbmd SET`

**Object type:** `parameter`

**Object:** `statement_timeout`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `SET`



#### `parameter statement_timeout → dbmd_acl_reader ALTER SYSTEM`

**Object type:** `parameter`

**Object:** `statement_timeout`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `ALTER SYSTEM`



#### `parameter work_mem → dbmd ALTER SYSTEM`

**Object type:** `parameter`

**Object:** `work_mem`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `ALTER SYSTEM`



#### `parameter work_mem → dbmd SET`

**Object type:** `parameter`

**Object:** `work_mem`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `SET`



#### `parameter work_mem → dbmd_acl_reader SET`

**Object type:** `parameter`

**Object:** `work_mem`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SET`



#### `procedure secure.clear_events() → PUBLIC EXECUTE`

**Object type:** `procedure`

**Object:** `secure.clear_events()`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `PUBLIC`

**Privilege:** `EXECUTE`



#### `procedure secure.clear_events() → dbmd_acl_owner EXECUTE`

**Object type:** `procedure`

**Object:** `secure.clear_events()`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `EXECUTE`



#### `procedure secure.clear_events() → dbmd_acl_reader EXECUTE`

**Object type:** `procedure`

**Object:** `secure.clear_events()`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `EXECUTE`



#### `schema public → PUBLIC USAGE`

**Object type:** `schema`

**Object:** `public`

**Grantor:** `pg_database_owner`

**Grantee:** `PUBLIC`

**Privilege:** `USAGE`



#### `schema public → pg_database_owner CREATE`

**Object type:** `schema`

**Object:** `public`

**Grantor:** `pg_database_owner`

**Grantee:** `pg_database_owner`

**Privilege:** `CREATE`



#### `schema public → pg_database_owner USAGE`

**Object type:** `schema`

**Object:** `public`

**Grantor:** `pg_database_owner`

**Grantee:** `pg_database_owner`

**Privilege:** `USAGE`



#### `schema secure → dbmd_acl_owner CREATE`

**Object type:** `schema`

**Object:** `secure`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `CREATE`



#### `schema secure → dbmd_acl_owner USAGE`

**Object type:** `schema`

**Object:** `secure`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `USAGE`



#### `schema secure → dbmd_acl_reader USAGE`

**Object type:** `schema`

**Object:** `secure`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`

**Grant option:** yes



#### `sequence secure.event_sequence → dbmd_acl_owner SELECT`

**Object type:** `sequence`

**Object:** `secure.event_sequence`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



#### `sequence secure.event_sequence → dbmd_acl_owner UPDATE`

**Object type:** `sequence`

**Object:** `secure.event_sequence`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



#### `sequence secure.event_sequence → dbmd_acl_owner USAGE`

**Object type:** `sequence`

**Object:** `secure.event_sequence`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `USAGE`



#### `sequence secure.event_sequence → dbmd_acl_reader SELECT`

**Object type:** `sequence`

**Object:** `secure.event_sequence`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



#### `sequence secure.event_sequence → dbmd_acl_reader UPDATE`

**Object type:** `sequence`

**Object:** `secure.event_sequence`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `UPDATE`



#### `sequence secure.event_sequence → dbmd_acl_reader USAGE`

**Object type:** `sequence`

**Object:** `secure.event_sequence`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



#### `foreign server secure_server → dbmd_acl_owner USAGE`

**Object type:** `foreign server`

**Object:** `secure_server`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `USAGE`



#### `foreign server secure_server → dbmd_acl_reader USAGE`

**Object type:** `foreign server`

**Object:** `secure_server`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



#### `table secure.events → dbmd_acl_owner DELETE`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `DELETE`



#### `table secure.events → dbmd_acl_owner INSERT`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `INSERT`



#### `table secure.events → dbmd_acl_owner MAINTAIN`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `MAINTAIN`



#### `table secure.events → dbmd_acl_owner REFERENCES`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `REFERENCES`



#### `table secure.events → dbmd_acl_owner SELECT`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



#### `table secure.events → dbmd_acl_owner TRIGGER`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRIGGER`



#### `table secure.events → dbmd_acl_owner TRUNCATE`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRUNCATE`



#### `table secure.events → dbmd_acl_owner UPDATE`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



#### `table secure.events → dbmd_acl_reader DELETE`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `DELETE`



#### `table secure.events → dbmd_acl_reader INSERT`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `INSERT`



#### `table secure.events → dbmd_acl_reader MAINTAIN`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `MAINTAIN`



#### `table secure.events → dbmd_acl_reader REFERENCES`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `REFERENCES`



#### `table secure.events → dbmd_acl_reader SELECT`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



#### `table secure.events → dbmd_acl_reader TRIGGER`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `TRIGGER`



#### `table secure.events → dbmd_acl_reader TRUNCATE`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `TRUNCATE`



#### `table secure.events → dbmd_acl_reader UPDATE`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `UPDATE`



#### `table column secure.events.payload → dbmd_acl_reader UPDATE`

**Object type:** `table column`

**Object:** `secure.events.payload`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `UPDATE`

**Grant option:** yes



#### `type secure.event_code → PUBLIC USAGE`

**Object type:** `type`

**Object:** `secure.event_code`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `PUBLIC`

**Privilege:** `USAGE`



#### `type secure.event_code → dbmd_acl_owner USAGE`

**Object type:** `type`

**Object:** `secure.event_code`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `USAGE`



#### `type secure.event_code → dbmd_acl_reader USAGE`

**Object type:** `type`

**Object:** `secure.event_code`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



#### `type secure.event_state → PUBLIC USAGE`

**Object type:** `type`

**Object:** `secure.event_state`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `PUBLIC`

**Privilege:** `USAGE`



#### `type secure.event_state → dbmd_acl_owner USAGE`

**Object type:** `type`

**Object:** `secure.event_state`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `USAGE`



#### `type secure.event_state → dbmd_acl_reader USAGE`

**Object type:** `type`

**Object:** `secure.event_state`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



#### `view secure.event_view → dbmd_acl_owner DELETE`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `DELETE`



#### `view secure.event_view → dbmd_acl_owner INSERT`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `INSERT`



#### `view secure.event_view → dbmd_acl_owner MAINTAIN`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `MAINTAIN`



#### `view secure.event_view → dbmd_acl_owner REFERENCES`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `REFERENCES`



#### `view secure.event_view → dbmd_acl_owner SELECT`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



#### `view secure.event_view → dbmd_acl_owner TRIGGER`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRIGGER`



#### `view secure.event_view → dbmd_acl_owner TRUNCATE`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRUNCATE`



#### `view secure.event_view → dbmd_acl_owner UPDATE`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



#### `view secure.event_view → dbmd_acl_reader SELECT`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



### Default Privileges

#### `dbmd_acl_owner / secure / sequences → dbmd_acl_reader USAGE`

**Owner:** `dbmd_acl_owner`

**Scope:** `secure`

**Object family:** `sequences`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



#### `dbmd_acl_owner / secure / types → dbmd_acl_reader USAGE`

**Owner:** `dbmd_acl_owner`

**Scope:** `secure`

**Object family:** `types`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



#### `dbmd_acl_owner / secure / routines → dbmd_acl_reader EXECUTE`

**Owner:** `dbmd_acl_owner`

**Scope:** `secure`

**Object family:** `routines`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `EXECUTE`



#### `dbmd_acl_owner / secure / tables → dbmd_acl_reader SELECT`

**Owner:** `dbmd_acl_owner`

**Scope:** `secure`

**Object family:** `tables`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`

**Grant option:** yes



#### `dbmd_acl_owner / database-wide / large objects → dbmd_acl_owner SELECT`

**Owner:** `dbmd_acl_owner`

**Scope:** `database-wide`

**Object family:** `large objects`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



#### `dbmd_acl_owner / database-wide / large objects → dbmd_acl_owner UPDATE`

**Owner:** `dbmd_acl_owner`

**Scope:** `database-wide`

**Object family:** `large objects`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



#### `dbmd_acl_owner / database-wide / large objects → dbmd_acl_reader SELECT`

**Owner:** `dbmd_acl_owner`

**Scope:** `database-wide`

**Object family:** `large objects`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



#### `dbmd_acl_owner / database-wide / schemas → dbmd_acl_owner CREATE`

**Owner:** `dbmd_acl_owner`

**Scope:** `database-wide`

**Object family:** `schemas`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `CREATE`



#### `dbmd_acl_owner / database-wide / schemas → dbmd_acl_owner USAGE`

**Owner:** `dbmd_acl_owner`

**Scope:** `database-wide`

**Object family:** `schemas`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `USAGE`



#### `dbmd_acl_owner / database-wide / schemas → dbmd_acl_reader USAGE`

**Owner:** `dbmd_acl_owner`

**Scope:** `database-wide`

**Object family:** `schemas`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



### Large Objects

#### `424242`

Fixture document payload

**Owner:** `dbmd_acl_owner`

**Contents:** `omitted`



### Collations

#### `temporal.unicode_fast`

Fast Unicode semantics

**Owner:** `dbmd`

**Provider:** `builtin`

**Deterministic:** yes

**Encoding:** `UTF8`

**Locale:** `PG_UNICODE_FAST`

**Version:** `1`



### Extensions

#### `btree_gist`

Temporal exclusion operator support

**Owner:** `dbmd`

**Schema:** `public`

**Version:** `1.8`

**Relocatable:** yes

**Owned objects:** 288 (`function`: 212, `operator`: 12, `operator class`: 26, `operator family`: 26, `type`: 12)



#### `plpgsql`

PL/pgSQL procedural language

**Owner:** `dbmd`

**Schema:** `pg_catalog`

**Version:** `1.0`

**Relocatable:** no

**Owned objects:** 4 (`function`: 3, `language`: 1)



#### `postgres_fdw`

foreign-data wrapper for remote PostgreSQL servers

**Owner:** `dbmd`

**Schema:** `public`

**Version:** `1.2`

**Relocatable:** yes

**Owned objects:** 6 (`foreign-data wrapper`: 1, `function`: 5)



## Source: `mysql_commerce` — `MySQL commerce`

Backend: `mysql`

### Schemas

| Name | Details |
|---|---|
| `test` | Default character set `utf8mb4`; collation `utf8mb4_0900_ai_ci`; encryption no; read-only no. |


### Tables

#### `test.accounts`

User accounts

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint unsigned` | no | - |  |
| `account_id` | `bigint unsigned` | no | - | `auto_increment` |
| `email` | `varchar(255)` | no | - |  |
| `normalized_email` | `varchar(255)` | yes | - | `STORED GENERATED`; generated as ``lower(`email`)`` |
| `secret_token` | `varchar(64)` | yes | - | `INVISIBLE`; invisible |
| `status` | `enum('active','disabled')` | no | `active` |  |
| `embedding` | `vector(3)` | yes | - |  |
| `default_embedding` | `vector(2048)` | yes | - |  |
| `home` | `point` | no | - | SRID 4326 |
| `updated_at` | `timestamp` | no | `CURRENT_TIMESTAMP` | `DEFAULT_GENERATED on update CURRENT_TIMESTAMP` |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `account_id` | - |
| `accounts_email_check` | `check` | `` | ``(`email` <> _latin1\'\')`` |
| `accounts_status_check` | `check` | `` | ``(`status` in (_latin1\'active\',_latin1\'disabled\'))``; not enforced |
| `accounts_tenant_email_uq` | `unique` | `tenant_id, email` | - |
| `accounts_tenant_fk` | `foreign_key` | `tenant_id` | references `test`.`tenants` (`tenant_id`); on update cascade; on delete restrict |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `account_id` | yes | BTREE | - |
| `accounts_email_desc_idx` | `email DESC` | no | BTREE; invisible; comment `Descending email lookup` | - |
| `accounts_email_ft` | `email` | no | FULLTEXT | - |
| `accounts_home_spatial` | `home(32)` | no | SPATIAL | - |
| `accounts_normalized_idx` | ``lower(`email`)`` | no | BTREE | - |
| `accounts_tenant_email_uq` | `tenant_id, email(120)` | yes | BTREE | - |


##### MySQL

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_0900_ai_ci`

**Create options:** `row_format=DYNAMIC`

```sql
CREATE TABLE `accounts` (
  `tenant_id` bigint unsigned NOT NULL,
  `account_id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `email` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL,
  `normalized_email` varchar(255) GENERATED ALWAYS AS (lower(`email`)) STORED,
  `secret_token` varchar(64) DEFAULT NULL /*!80023 INVISIBLE */,
  `status` enum('active','disabled') NOT NULL DEFAULT 'active',
  `embedding` vector(3) DEFAULT NULL,
  `default_embedding` vector(2048) DEFAULT NULL,
  `home` point NOT NULL /*!80003 SRID 4326 */,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`account_id`),
  UNIQUE KEY `accounts_tenant_email_uq` (`tenant_id`,`email`(120)),
  KEY `accounts_email_desc_idx` (`email` DESC) COMMENT 'Descending email lookup' /*!80000 INVISIBLE */,
  SPATIAL KEY `accounts_home_spatial` (`home`),
  KEY `accounts_normalized_idx` ((lower(`email`))),
  FULLTEXT KEY `accounts_email_ft` (`email`),
  CONSTRAINT `accounts_tenant_fk` FOREIGN KEY (`tenant_id`) REFERENCES `tenants` (`tenant_id`) ON DELETE RESTRICT ON UPDATE CASCADE,
  CONSTRAINT `accounts_email_check` CHECK ((`email` <> _latin1'')),
  CONSTRAINT `accounts_status_check` CHECK ((`status` in (_latin1'active',_latin1'disabled'))) /*!80016 NOT ENFORCED */
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci ROW_FORMAT=DYNAMIC COMMENT='User accounts'
```


#### `test.generated_primary_key`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `my_row_id` | `bigint unsigned` | no | - | `auto_increment INVISIBLE`; invisible |
| `payload` | `varchar(64)` | yes | - |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `my_row_id` | - |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `my_row_id` | yes | BTREE | - |


##### MySQL

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE TABLE `generated_primary_key` (
  `my_row_id` bigint unsigned NOT NULL AUTO_INCREMENT /*!80023 INVISIBLE */,
  `payload` varchar(64) DEFAULT NULL,
  PRIMARY KEY (`my_row_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci
```


#### `test.inline_memberships`

Exercises MySQL 9 inline implicit-parent foreign keys

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `membership_id` | `bigint unsigned` | no | - | `auto_increment` |
| `tenant_id` | `bigint unsigned` | no | - |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `membership_id` | - |
| `inline_memberships_ibfk_1` | `foreign_key` | `tenant_id` | references `test`.`tenants` (`tenant_id`); on update no action; on delete cascade |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `membership_id` | yes | BTREE | - |
| `tenant_id` | `tenant_id` | no | BTREE | - |


##### MySQL

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE TABLE `inline_memberships` (
  `membership_id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint unsigned NOT NULL,
  PRIMARY KEY (`membership_id`),
  KEY `tenant_id` (`tenant_id`),
  CONSTRAINT `inline_memberships_ibfk_1` FOREIGN KEY (`tenant_id`) REFERENCES `tenants` (`tenant_id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Exercises MySQL 9 inline implicit-parent foreign keys'
```


#### `test.memory_lookup`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `lookup_key` | `varchar(64)` | no | - |  |
| `payload` | `varchar(255)` | yes | - |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `lookup_key` | - |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `lookup_key` | yes | HASH | - |
| `memory_payload_hash` | `payload` | no | HASH | - |


##### MySQL

**Engine:** `MEMORY`

**Row format:** `Fixed`

**Collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE TABLE `memory_lookup` (
  `lookup_key` varchar(64) NOT NULL,
  `payload` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`lookup_key`),
  KEY `memory_payload_hash` (`payload`) USING HASH
) ENGINE=MEMORY DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci
```


#### `test.monthly_metrics`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `occurred_on` | `date` | no | - |  |
| `metric` | `varchar(64)` | no | - |  |
| `value` | `bigint` | no | - |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `occurred_on, metric` | - |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `occurred_on, metric` | yes | BTREE | - |


##### MySQL

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_0900_ai_ci`

**Create options:** `partitioned`

**Subpartition:** `p2025_h0`: method=`HASH`, expression=``month(`occurred_on`)``, boundary=`2026`, position=1; nodegroup=`default`

**Subpartition:** `p2025_h1`: method=`HASH`, expression=``month(`occurred_on`)``, boundary=`2026`, position=2; nodegroup=`default`

**Subpartition:** `pmax_h0`: method=`HASH`, expression=``month(`occurred_on`)``, boundary=`MAXVALUE`, position=1; nodegroup=`default`

**Subpartition:** `pmax_h1`: method=`HASH`, expression=``month(`occurred_on`)``, boundary=`MAXVALUE`, position=2; nodegroup=`default`

```sql
CREATE TABLE `monthly_metrics` (
  `occurred_on` date NOT NULL,
  `metric` varchar(64) NOT NULL,
  `value` bigint NOT NULL,
  PRIMARY KEY (`occurred_on`,`metric`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci
/*!50100 PARTITION BY RANGE (year(`occurred_on`))
SUBPARTITION BY HASH (month(`occurred_on`))
(PARTITION p2025 VALUES LESS THAN (2026)
 (SUBPARTITION p2025_h0 ENGINE = InnoDB,
  SUBPARTITION p2025_h1 ENGINE = InnoDB),
 PARTITION pmax VALUES LESS THAN MAXVALUE
 (SUBPARTITION pmax_h0 ENGINE = InnoDB,
  SUBPARTITION pmax_h1 ENGINE = InnoDB)) */
```


#### `test.tenants`

Application tenants

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint unsigned` | no | - | `auto_increment` |
| `name` | `varchar(120)` | no | - | Tenant display name |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `tenant_id` | - |
| `tenants_name_uq` | `unique` | `name` | - |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `tenant_id` | yes | BTREE | - |
| `tenants_name_uq` | `name` | yes | BTREE | - |


##### MySQL

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE TABLE `tenants` (
  `tenant_id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(120) NOT NULL COMMENT 'Tenant display name',
  PRIMARY KEY (`tenant_id`),
  UNIQUE KEY `tenants_name_uq` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='Application tenants'
```


### Views

#### `test.active_accounts`

**Kind:** `sql`

**Check option:** cascaded

**Updatable:** yes

**Security:** invoker

**Definer:** `root@localhost`

| Column | Type | Nullable |
|---|---|---|


```sql
CREATE ALGORITHM=UNDEFINED DEFINER=`root`@`localhost` SQL SECURITY INVOKER VIEW `active_accounts` AS select `accounts`.`tenant_id` AS `tenant_id`,`accounts`.`account_id` AS `account_id`,`accounts`.`email` AS `email` from `accounts` where (`accounts`.`status` = 'active') WITH CASCADED CHECK OPTION
```

#### `test.tenant_documents`

**Kind:** `json_relational_duality`

**Check option:** none

**Updatable:** yes

**Security:** definer

**Definer:** `root@localhost`

**JSON column:** `data`

**Root table:** `test.tenants`

**Status:** `valid`

**Operations:** `insert=false, update=false, delete=false, read_only=true`

**Mapped table:** `#0 test.tenants parent=None relationship=- where=- permissions=false/false/false read_only=true root=true`

**JSON field:** `_id -> #0 test.tenants.tenant_id, permissions=false/false/false read_only=true root=true`

**JSON field:** `name -> #0 test.tenants.name, permissions=false/false/false read_only=true root=true`

| Column | Type | Nullable |
|---|---|---|


```sql
CREATE ALGORITHM=UNDEFINED DEFINER=`root`@`localhost` SQL SECURITY DEFINER JSON RELATIONAL DUALITY VIEW `tenant_documents` AS select json_duality_object('_id':`tenants`.`tenant_id`,'name':`tenants`.`name`) AS `JSON_DUALITY_OBJECT('_id':tenant_id, 'name':name)` from `tenants`
```

### Triggers

#### `test.accounts_updated`

**`before` `update`** on `test.accounts`.

**Orientation:** for each row

**Order:** 1

**Definer:** `root@localhost`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` TRIGGER `accounts_updated` BEFORE UPDATE ON `accounts` FOR EACH ROW SET NEW.updated_at = CURRENT_TIMESTAMP
```

#### `test.accounts_update_marker`

**`before` `update`** on `test.accounts`.

**Orientation:** for each row

**Order:** 2

**Definer:** `root@localhost`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` TRIGGER `accounts_update_marker` BEFORE UPDATE ON `accounts` FOR EACH ROW SET @dbmd_last_account = NEW.account_id
```

### Routines

#### `test.disable_account`

**Kind:** procedure

**Data access:** modifies SQL data

**Deterministic:** no

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

**Parameters:** `in target_id bigint unsigned`

**External language:** `SQL`

```sql
CREATE DEFINER=`root`@`localhost` PROCEDURE `disable_account`(IN target_id BIGINT UNSIGNED)
    MODIFIES SQL DATA
UPDATE accounts SET status = 'disabled' WHERE account_id = target_id
```


#### `test.next_account_id`

**Kind:** procedure

**Data access:** no SQL

**Deterministic:** no

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

**Parameters:** `in current_id bigint unsigned, out next_id bigint unsigned`

**External language:** `SQL`

```sql
CREATE DEFINER=`root`@`localhost` PROCEDURE `next_account_id`(IN current_id BIGINT UNSIGNED, OUT next_id BIGINT UNSIGNED)
    NO SQL
SET next_id = current_id + 1
```


#### `test.normalize_email`

**Kind:** function

**Data access:** no SQL

**Deterministic:** yes

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

**Parameters:** `return varchar(255), in value varchar(255)`

**External language:** `SQL`

```sql
CREATE DEFINER=`root`@`localhost` FUNCTION `normalize_email`(value VARCHAR(255)) RETURNS varchar(255) CHARSET utf8mb4
    NO SQL
    DETERMINISTIC
RETURN lower(value)
```


### Events

#### `test.archive_accounts_once`

**Definer:** `root@localhost`

**Type:** one time

**Status:** disabled

**Time zone:** `SYSTEM`

**On completion:** preserve

**Schedule:** `AT 2031-01-01 00:00:00`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Originator server ID:** 1

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` EVENT `archive_accounts_once` ON SCHEDULE AT '2031-01-01 00:00:00' ON COMPLETION PRESERVE DISABLE DO SET @dbmd_archive_requested = 1
```


#### `test.purge_disabled_accounts`

Remove old disabled accounts

**Definer:** `root@localhost`

**Type:** recurring

**Status:** disabled

**Time zone:** `SYSTEM`

**On completion:** preserve

**Schedule:** `EVERY 1 DAY STARTS 2030-01-01 00:00:00`

**SQL mode:** `ONLY_FULL_GROUP_BY,STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION`

**Originator server ID:** 1

**Character set client:** `latin1`

**Connection collation:** `latin1_swedish_ci`

**Database collation:** `utf8mb4_0900_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` EVENT `purge_disabled_accounts` ON SCHEDULE EVERY 1 DAY STARTS '2030-01-01 00:00:00' ON COMPLETION PRESERVE DISABLE COMMENT 'Remove old disabled accounts' DO DELETE FROM accounts WHERE status = 'disabled' AND updated_at < CURRENT_TIMESTAMP - INTERVAL 365 DAY
```


## Source: `mariadb_commerce` — `MariaDB commerce`

Backend: `mariadb`

### Schemas

| Name | Details |
|---|---|
| `test` | Default character set `utf8mb4`; collation `utf8mb4_uca1400_ai_ci`. Commerce schema fixture |


### Tables

#### `test.accounts`

Versioned user accounts

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint(20) unsigned` | no | - |  |
| `account_id` | `bigint(20) unsigned` | no | - | `auto_increment` |
| `email` | `varchar(255)` | no | - |  |
| `normalized_email` | `varchar(255)` | yes | `NULL` | `STORED GENERATED`; stored generated as ``lcase(`email`)`` |
| `profile_document` | `xmltype` | yes | `NULL` | MariaDB 12.3 XML profile payload |
| `status` | `enum('active','disabled')` | no | `'active'` |  |
| `secret_token` | `varchar(64)` | yes | `NULL` | `INVISIBLE`; invisible |
| `home` | `point` | no | - |  |
| `row_start` | `timestamp(6)` | no | - | `STORED GENERATED`; stored generated as `ROW START`; system-time period start |
| `row_end` | `timestamp(6)` | no | - | `STORED GENERATED`; stored generated as `ROW END`; system-time period end |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `account_id, row_end` | - |
| `accounts_email_check` | `check` | `` | `` `email` <> '' ``; declared at table level |
| `accounts_tenant_email_uq` | `unique` | `tenant_id, email, row_end` | - |
| `accounts_tenant_fk` | `foreign_key` | `tenant_id` | references `test`.`tenants` (`tenant_id`); on update cascade; on delete restrict |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `account_id, row_end` | yes | BTREE | - |
| `accounts_email_fulltext` | `email` | no | FULLTEXT | - |
| `accounts_email_ignored_idx` | `email` | no | BTREE; ignored | - |
| `accounts_home_spatial` | `home(32)` | no | SPATIAL | - |
| `accounts_status_desc_idx` | `status DESC` | no | BTREE; comment `Status lookup ordering` | - |
| `accounts_tenant_email_uq` | `tenant_id, email(120), row_end` | yes | BTREE | - |


##### MariaDB

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_uca1400_ai_ci`

**System versioning:** enabled

**System-time period:** `row_start, row_end`

```sql
CREATE TABLE `accounts` (
  `tenant_id` bigint(20) unsigned NOT NULL,
  `account_id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `email` varchar(255) NOT NULL,
  `normalized_email` varchar(255) GENERATED ALWAYS AS (lcase(`email`)) STORED,
  `profile_document` xmltype DEFAULT NULL COMMENT 'MariaDB 12.3 XML profile payload',
  `status` enum('active','disabled') NOT NULL DEFAULT 'active',
  `secret_token` varchar(64) DEFAULT NULL INVISIBLE,
  `home` point NOT NULL,
  `row_start` timestamp(6) GENERATED ALWAYS AS ROW START,
  `row_end` timestamp(6) GENERATED ALWAYS AS ROW END,
  PRIMARY KEY (`account_id`,`row_end`),
  UNIQUE KEY `accounts_tenant_email_uq` (`tenant_id`,`email`(120),`row_end`),
  KEY `accounts_status_desc_idx` (`status` DESC) COMMENT 'Status lookup ordering',
  KEY `accounts_email_ignored_idx` (`email`) IGNORED,
  SPATIAL KEY `accounts_home_spatial` (`home`),
  FULLTEXT KEY `accounts_email_fulltext` (`email`),
  PERIOD FOR SYSTEM_TIME (`row_start`, `row_end`),
  CONSTRAINT `accounts_tenant_fk` FOREIGN KEY (`tenant_id`) REFERENCES `tenants` (`tenant_id`) ON UPDATE CASCADE,
  CONSTRAINT `accounts_email_check` CHECK (`email` <> '')
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci COMMENT='Versioned user accounts' WITH SYSTEM VERSIONING
```


#### `test.discarded_events`

Exercises an installed storage-engine plugin

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `event_id` | `bigint(20)` | no | - |  |


##### MariaDB

**Engine:** `BLACKHOLE`

**Row format:** `Fixed`

**Collation:** `utf8mb4_uca1400_ai_ci`

```sql
CREATE TABLE `discarded_events` (
  `event_id` bigint(20) NOT NULL
) ENGINE=BLACKHOLE DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci COMMENT='Exercises an installed storage-engine plugin'
```


#### `test.monthly_metrics`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `occurred_on` | `date` | no | - |  |
| `metric` | `varchar(64)` | no | - |  |
| `value` | `bigint(20)` | no | - |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `occurred_on, metric` | - |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `occurred_on, metric` | yes | BTREE | - |


##### MariaDB

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_uca1400_ai_ci`

**Create options:** `partitioned`

**Subpartition:** `p2025_h0`: method=`hash`, expression=``month(`occurred_on`)``, boundary=`2026`, position=1; nodegroup=`default`

**Subpartition:** `p2025_h1`: method=`hash`, expression=``month(`occurred_on`)``, boundary=`2026`, position=2; nodegroup=`default`

**Subpartition:** `pmax_h0`: method=`hash`, expression=``month(`occurred_on`)``, boundary=`MAXVALUE`, position=1; nodegroup=`default`

**Subpartition:** `pmax_h1`: method=`hash`, expression=``month(`occurred_on`)``, boundary=`MAXVALUE`, position=2; nodegroup=`default`

```sql
CREATE TABLE `monthly_metrics` (
  `occurred_on` date NOT NULL,
  `metric` varchar(64) NOT NULL,
  `value` bigint(20) NOT NULL,
  PRIMARY KEY (`occurred_on`,`metric`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci
 PARTITION BY RANGE (year(`occurred_on`))
SUBPARTITION BY HASH (month(`occurred_on`))
(PARTITION `p2025` VALUES LESS THAN (2026)
 (SUBPARTITION `p2025_h0` ENGINE = InnoDB,
  SUBPARTITION `p2025_h1` ENGINE = InnoDB),
 PARTITION `pmax` VALUES LESS THAN MAXVALUE
 (SUBPARTITION `pmax_h0` ENGINE = InnoDB,
  SUBPARTITION `pmax_h1` ENGINE = InnoDB))
```


#### `test.tenants`

Application tenants

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint(20) unsigned` | no | - | `auto_increment` |
| `name` | `varchar(120)` | no | - | Tenant display name |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `tenant_id` | - |
| `tenants_name_uq` | `unique` | `name` | - |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `tenant_id` | yes | BTREE | - |
| `tenants_name_uq` | `name` | yes | BTREE | - |


##### MariaDB

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_uca1400_ai_ci`

```sql
CREATE TABLE `tenants` (
  `tenant_id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(120) NOT NULL COMMENT 'Tenant display name',
  PRIMARY KEY (`tenant_id`),
  UNIQUE KEY `tenants_name_uq` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci COMMENT='Application tenants'
```


#### `test.tenant_audits`

Reuses a foreign-key name in the same schema

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `audit_id` | `bigint(20) unsigned` | no | - | `auto_increment` |
| `tenant_id` | `bigint(20) unsigned` | no | - |  |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `PRIMARY` | `primary_key` | `audit_id` | - |
| `accounts_tenant_fk` | `foreign_key` | `tenant_id` | references `test`.`tenants` (`tenant_id`); on update restrict; on delete cascade |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `PRIMARY` | `audit_id` | yes | BTREE | - |
| `accounts_tenant_fk` | `tenant_id` | no | BTREE | - |


##### MariaDB

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_uca1400_ai_ci`

```sql
CREATE TABLE `tenant_audits` (
  `audit_id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint(20) unsigned NOT NULL,
  PRIMARY KEY (`audit_id`),
  KEY `accounts_tenant_fk` (`tenant_id`),
  CONSTRAINT `accounts_tenant_fk` FOREIGN KEY (`tenant_id`) REFERENCES `tenants` (`tenant_id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci COMMENT='Reuses a foreign-key name in the same schema'
```


#### `test.tenant_embeddings`

Bitemporal tenant embeddings

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint(20) unsigned` | no | - |  |
| `valid_from` | `date` | no | - |  |
| `valid_to` | `date` | no | - |  |
| `embedding` | `vector(5)` | no | - |  |
| `row_start` | `timestamp(6)` | no | - | `STORED GENERATED`; stored generated as `ROW START`; system-time period start |
| `row_end` | `timestamp(6)` | no | - | `STORED GENERATED`; stored generated as `ROW END`; system-time period end |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `tenant_validity_uq` | `unique` | `tenant_id, row_end, valid_to, valid_from` | period `validity` without overlaps |
| `validity` | `check` | `` | `` `valid_from` < `valid_to` ``; declared at table level |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `embedding_vector_idx` | `embedding` | no | VECTOR; M=8; distance=cosine | - |
| `tenant_validity_uq` | `tenant_id, row_end, valid_to, valid_from` | yes | BTREE; period validity without overlaps | - |


##### MariaDB

**Engine:** `InnoDB`

**Row format:** `Dynamic`

**Collation:** `utf8mb4_uca1400_ai_ci`

**System versioning:** enabled

**System-time period:** `row_start, row_end`

**Application-time period:** `validity`: `valid_from` to `valid_to`

```sql
CREATE TABLE `tenant_embeddings` (
  `tenant_id` bigint(20) unsigned NOT NULL,
  `valid_from` date NOT NULL,
  `valid_to` date NOT NULL,
  `embedding` vector(5) NOT NULL,
  `row_start` timestamp(6) GENERATED ALWAYS AS ROW START,
  `row_end` timestamp(6) GENERATED ALWAYS AS ROW END,
  PERIOD FOR `validity` (`valid_from`, `valid_to`),
  UNIQUE KEY `tenant_validity_uq` (`tenant_id`,`row_end`,`validity` WITHOUT OVERLAPS),
  VECTOR KEY `embedding_vector_idx` (`embedding`) `M`=8 `DISTANCE`=cosine,
  PERIOD FOR SYSTEM_TIME (`row_start`, `row_end`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci COMMENT='Bitemporal tenant embeddings' WITH SYSTEM VERSIONING
```


### Views

#### `test.active_accounts`

**Check option:** cascaded

**Updatable:** yes

**Security:** invoker

**Algorithm:** merge

**Definer:** `root@localhost`

| Column | Type | Nullable |
|---|---|---|


```sql
CREATE ALGORITHM=MERGE DEFINER=`root`@`localhost` SQL SECURITY INVOKER VIEW `active_accounts` AS select `accounts`.`tenant_id` AS `tenant_id`,`accounts`.`account_id` AS `account_id`,`accounts`.`email` AS `email` from `accounts` where `accounts`.`status` = 'active' WITH CASCADED CHECK OPTION
```

### Triggers

#### `test.accounts_changed`

**`after` `insert, update, delete`** on `test.accounts`.

**Orientation:** for each row

**Order:** 1

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` TRIGGER accounts_changed
AFTER INSERT OR UPDATE OR DELETE ON accounts
FOR EACH ROW SET @dbmd_mariadb_account_changed = 1
```

#### `test.accounts_updated`

**`before` `update`** on `test.accounts`.

**Orientation:** for each row

**Order:** 1

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Update columns:** `email, status`

```sql
CREATE DEFINER=`root`@`localhost` TRIGGER accounts_updated
BEFORE UPDATE OF status, email ON accounts
FOR EACH ROW SET NEW.status = COALESCE(NEW.status, OLD.status)
```

#### `test.accounts_update_marker`

**`before` `update`** on `test.accounts`.

**Orientation:** for each row

**Order:** 2

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

```sql
CREATE DEFINER=`root`@`localhost` TRIGGER accounts_update_marker
BEFORE UPDATE ON accounts
FOR EACH ROW
SET @dbmd_mariadb_last_account = NEW.account_id
```

### Routines, Sequences, and Events

#### `test.disable_account`

**Kind:** procedure

**Data access:** modifies SQL data

**Deterministic:** no

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Parameters:** `in target_id bigint(20) unsigned`

```sql
CREATE DEFINER=`root`@`localhost` PROCEDURE `disable_account`(IN target_id BIGINT UNSIGNED)
    MODIFIES SQL DATA
UPDATE accounts SET status = 'disabled' WHERE account_id = target_id
```


#### `test.next_account_id`

**Kind:** procedure

**Data access:** no SQL

**Deterministic:** no

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Parameters:** `in current_id bigint(20) unsigned, out next_id bigint(20) unsigned`

```sql
CREATE DEFINER=`root`@`localhost` PROCEDURE `next_account_id`(IN current_id BIGINT UNSIGNED, OUT next_id BIGINT UNSIGNED)
    NO SQL
SET next_id = current_id + 1
```


#### `test.normalize_email`

**Kind:** function

**Data access:** no SQL

**Deterministic:** yes

**Security:** definer

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Parameters:** `return varchar(255), in value varchar(255) default 'fallback@example.invalid'`

```sql
CREATE DEFINER=`root`@`localhost` FUNCTION `normalize_email`(value VARCHAR(255) DEFAULT 'fallback@example.invalid') RETURNS varchar(255) CHARSET utf8mb4 COLLATE utf8mb4_uca1400_ai_ci
    NO SQL
    DETERMINISTIC
RETURN lower(value)
```


#### `test.descending_order_seq`

**Kind:** `sequence`

**Type:** `bigint`

**Start:** `0`

**Minimum:** `-20`

**Maximum:** `0`

**Increment:** `-2`

**Cycle:** yes

**Cache:** `0`

**Engine:** `InnoDB`

```sql
CREATE SEQUENCE `descending_order_seq` start with 0 minvalue -20 maxvalue 0 increment by -2 nocache cycle ENGINE=InnoDB
```


#### `test.order_number_seq`

**Kind:** `sequence`

**Type:** `bigint`

**Start:** `1000`

**Minimum:** `1`

**Maximum:** `9223372036854775806`

**Increment:** `10`

**Cycle:** no

**Cache:** `20`

**Engine:** `InnoDB`

```sql
CREATE SEQUENCE `order_number_seq` start with 1000 minvalue 1 maxvalue 9223372036854775806 increment by 10 cache 20 nocycle ENGINE=InnoDB
```


#### `test.archive_accounts_once`

**Kind:** `event`

**Status:** disabled

**Schedule:** one time

**Completion:** preserve

**Definer:** `root@localhost`

**Time zone:** `SYSTEM`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Originator:** 1

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Execute at:** `2031-01-01 00:00:00`

```sql
CREATE DEFINER=`root`@`localhost` EVENT `archive_accounts_once` ON SCHEDULE AT '2031-01-01 00:00:00' ON COMPLETION PRESERVE DISABLE DO SET @dbmd_mariadb_archive_requested = 1
```


#### `test.purge_disabled_accounts`

Remove old disabled accounts

**Kind:** `event`

**Status:** disabled

**Schedule:** recurring

**Completion:** preserve

**Definer:** `root@localhost`

**Time zone:** `SYSTEM`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Originator:** 1

**Character set client:** `utf8mb4`

**Connection collation:** `utf8mb4_uca1400_ai_ci`

**Database collation:** `utf8mb4_uca1400_ai_ci`

**Interval:** `1` day

**Starts:** `2030-01-01 00:00:00`

```sql
CREATE DEFINER=`root`@`localhost` EVENT `purge_disabled_accounts` ON SCHEDULE EVERY 1 DAY STARTS '2030-01-01 00:00:00' ON COMPLETION PRESERVE DISABLE COMMENT 'Remove old disabled accounts' DO DELETE FROM accounts WHERE status = 'disabled' AND row_end < CURRENT_TIMESTAMP - INTERVAL 365 DAY
```


#### `analytics_remote`

**Kind:** `server`

**Wrapper:** `mariadb`

**Host:** `db.internal`

**Database:** `analytics`

**Username:** `reader`

**Owner:** `platform`

**Port:** `3307`

**Option:** `DATABASE`: `analytics`

**Option:** `HOST`: `db.internal`

**Option:** `OWNER`: `platform`

**Option:** `PASSWORD`: [redacted]

**Option:** `PORT`: `3307`

**Option:** `REGION`: `eu-west-1`

**Option:** `USER`: `reader`



#### `Aria (storage engine)`

Crash-safe tables with MyISAM heritage. Used for internal temporary tables and privilege tables

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.6`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.6`

**Author:** `MariaDB Corporation Ab`



#### `associative_array (data type)`

Data type ASSOCIATIVE_ARRAY

**Kind:** `plugin`

**Type:** data type

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Rakuten Securities`



#### `binlog (daemon)`

This is a plugin to represent the binlog in a transaction

**Kind:** `plugin`

**Type:** daemon

**Version:** `2.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `2.0`

**Author:** `MySQL AB`



#### `BLACKHOLE (storage engine)`

/dev/null storage engine (anything you write to it disappears)

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Library:** `ha_blackhole.so`

**Library version:** `1.15`

**Authentication version:** `1.0`

**Author:** `MySQL AB`



#### `caching_sha2_password (authentication)`

MySQL-compatible SHA2 authentication

**Kind:** `plugin`

**Type:** authentication

**Version:** `1.0`

**Status:** active

**Type version:** `2.3`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Library:** `auth_mysql_sha2.so`

**Library version:** `1.15`

**Authentication version:** `1.0`

**Author:** `Oracle Corporation, Sergei Golubchik`



#### `CLIENT_STATISTICS (information schema)`

Client Statistics

**Kind:** `plugin`

**Type:** information schema

**Version:** `2.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `2.0`

**Author:** `Percona and Sergei Golubchik`



#### `CSV (storage engine)`

Stores tables as CSV files

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Brian Aker, MySQL AB`



#### `FEEDBACK (information schema)`

MariaDB User Feedback Plugin

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.1`

**Status:** disabled

**Type version:** `120302.0`

**License:** GPL

**Load option:** off

**Maturity:** stable

**Authentication version:** `1.1`

**Author:** `Sergei Golubchik`



#### `GEOMETRY_COLUMNS (information schema)`

Lists all geometry columns

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB`



#### `INDEX_STATISTICS (information schema)`

Index Statistics

**Kind:** `plugin`

**Type:** information schema

**Version:** `2.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `2.0`

**Author:** `Percona and Sergei Golubchik`



#### `inet4 (data type)`

Data type INET4

**Kind:** `plugin`

**Type:** data type

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0.1`

**Author:** `MariaDB Corporation`



#### `inet6 (data type)`

Data type INET6

**Kind:** `plugin`

**Type:** data type

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `inet6_aton (native function)`

Function INET6_ATON()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `inet6_ntoa (native function)`

Function INET6_NTOA()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `inet_aton (native function)`

Function INET_ATON()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `inet_ntoa (native function)`

Function INET_NTOA()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `InnoDB (storage engine)`

Supports transactions, row-level locking, foreign keys and encryption for tables

**Kind:** `plugin`

**Type:** storage engine

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_BUFFER_PAGE (information schema)`

InnoDB Buffer Page Information

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_BUFFER_PAGE_LRU (information schema)`

InnoDB Buffer Page in LRU

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_BUFFER_POOL_STATS (information schema)`

InnoDB Buffer Pool Statistics Information

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_CMP (information schema)`

Statistics for the InnoDB compression

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_CMPMEM (information schema)`

Statistics for the InnoDB compressed buffer pool

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_CMPMEM_RESET (information schema)`

Statistics for the InnoDB compressed buffer pool; reset cumulated counts

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_CMP_PER_INDEX (information schema)`

Statistics for the InnoDB compression (per index)

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_CMP_PER_INDEX_RESET (information schema)`

Statistics for the InnoDB compression (per index); reset cumulated counts

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_CMP_RESET (information schema)`

Statistics for the InnoDB compression; reset cumulated counts

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_FT_BEING_DELETED (information schema)`

INNODB AUXILIARY FTS BEING DELETED TABLE

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_FT_CONFIG (information schema)`

INNODB AUXILIARY FTS CONFIG TABLE

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_FT_DEFAULT_STOPWORD (information schema)`

Default stopword list for InnoDB Full Text Search

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_FT_DELETED (information schema)`

INNODB AUXILIARY FTS DELETED TABLE

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_FT_INDEX_CACHE (information schema)`

INNODB AUXILIARY FTS INDEX CACHED

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_FT_INDEX_TABLE (information schema)`

INNODB AUXILIARY FTS INDEX TABLE

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_LOCKS (information schema)`

InnoDB conflicting locks

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_LOCK_WAITS (information schema)`

InnoDB which lock is blocking which

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_METRICS (information schema)`

InnoDB Metrics Info

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_SYS_COLUMNS (information schema)`

InnoDB SYS_COLUMNS

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_SYS_FIELDS (information schema)`

InnoDB SYS_FIELDS

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_SYS_FOREIGN (information schema)`

InnoDB SYS_FOREIGN

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_SYS_FOREIGN_COLS (information schema)`

InnoDB SYS_FOREIGN_COLS

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_SYS_INDEXES (information schema)`

InnoDB SYS_INDEXES

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_SYS_TABLES (information schema)`

InnoDB SYS_TABLES

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_SYS_TABLESPACES (information schema)`

InnoDB tablespaces

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_SYS_TABLESTATS (information schema)`

InnoDB SYS_TABLESTATS

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_SYS_VIRTUAL (information schema)`

InnoDB SYS_VIRTUAL

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `INNODB_TABLESPACES_ENCRYPTION (information schema)`

InnoDB TABLESPACES_ENCRYPTION

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** BSD

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Google Inc`



#### `INNODB_TRX (information schema)`

InnoDB transactions

**Kind:** `plugin`

**Type:** information schema

**Version:** `12.3`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `12.3.2`

**Author:** `Oracle Corporation`



#### `is_ipv4 (native function)`

Function IS_IPV4()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `is_ipv4_compat (native function)`

Function IS_IPV4_COMPAT()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `is_ipv4_mapped (native function)`

Function IS_IPV4_MAPPED()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `is_ipv6 (native function)`

Function IS_IPV6()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `MEMORY (storage engine)`

Hash based, stored in memory, useful for temporary tables

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MySQL AB`



#### `mhnsw (daemon)`

A plugin for mhnsw vector index algorithm

**Kind:** `plugin`

**Type:** daemon

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB plc`



#### `MRG_MyISAM (storage engine)`

Collection of identical MyISAM tables

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MySQL AB`



#### `MyISAM (storage engine)`

Non-transactional engine with good performance and small data footprint

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MySQL AB`



#### `mysql_native_password (authentication)`

Native MySQL authentication

**Kind:** `plugin`

**Type:** authentication

**Version:** `1.0`

**Status:** active

**Type version:** `2.3`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `R.J.Silk, Sergei Golubchik`



#### `mysql_old_password (authentication)`

Old MySQL-4.0 authentication

**Kind:** `plugin`

**Type:** authentication

**Version:** `1.0`

**Status:** active

**Type version:** `2.3`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `R.J.Silk, Sergei Golubchik`



#### `online_alter_log (daemon)`

This is a plugin to represent the online alter log in a transaction

**Kind:** `plugin`

**Type:** daemon

**Version:** `2.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `2.0`

**Author:** `MariaDB PLC`



#### `partition (storage engine)`

Partition Storage Engine Helper

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Mikael Ronstrom, MySQL AB`



#### `PERFORMANCE_SCHEMA (storage engine)`

Performance Schema

**Kind:** `plugin`

**Type:** storage engine

**Version:** `0.1`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `5.7.31`

**Author:** `Marc Alff, Oracle`



#### `SEQUENCE (storage engine)`

Generated tables filled with sequential values

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `0.1`

**Author:** `Sergei Golubchik`



#### `SPATIAL_REF_SYS (information schema)`

Lists all geometry columns

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB`



#### `SQL_SEQUENCE (storage engine)`

Sequence Storage Engine for CREATE SEQUENCE

**Kind:** `plugin`

**Type:** storage engine

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `jianwei.zhao @ Aliyun & Monty @ MariaDB corp`



#### `sys_guid (native function)`

Function SYS_GUID()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `sys_refcursor (data type)`

Data type SYS_REFCURSOR

**Kind:** `plugin`

**Type:** data type

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `TABLE_STATISTICS (information schema)`

Table Statistics

**Kind:** `plugin`

**Type:** information schema

**Version:** `2.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `2.0`

**Author:** `Percona and Sergei Golubchik`



#### `THREAD_POOL_GROUPS (information schema)`

Provides information about threadpool groups.

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Vladislav Vaintroub`



#### `THREAD_POOL_QUEUES (information schema)`

Provides information about threadpool queues.

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Vladislav Vaintroub`



#### `THREAD_POOL_STATS (information schema)`

Provides performance counter information for threadpool.

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Vladislav Vaintroub`



#### `THREAD_POOL_WAITS (information schema)`

Provides wait counters for threadpool.

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Vladislav Vaintroub`



#### `unix_socket (authentication)`

Unix Socket based authentication

**Kind:** `plugin`

**Type:** authentication

**Version:** `1.1`

**Status:** active

**Type version:** `2.3`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.1`

**Author:** `Sergei Golubchik`



#### `USER_STATISTICS (information schema)`

User Statistics

**Kind:** `plugin`

**Type:** information schema

**Version:** `2.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `2.0`

**Author:** `Percona and Sergei Golubchik`



#### `user_variables (information schema)`

User-defined variables

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Sergey Vojtovich`



#### `uuid (data type)`

Data type UUID

**Kind:** `plugin`

**Type:** data type

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `uuid (native function)`

Function UUID()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `uuid_v4 (native function)`

Function UUID_v4()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0.1`

**Author:** `Stefano Petrilli`



#### `uuid_v7 (native function)`

Function UUID_v7()

**Kind:** `plugin`

**Type:** native function

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0.1`

**Author:** `Stefano Petrilli`



#### `wsrep (replication)`

Wsrep replication plugin

**Kind:** `plugin`

**Type:** replication

**Version:** `1.0`

**Status:** active

**Type version:** `2.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `Codership Oy`



#### `WSREP_MEMBERSHIP (information schema)`

Information about group members

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Library:** `wsrep_info.so`

**Library version:** `1.15`

**Authentication version:** `1.0`

**Author:** `Nirbhay Choubey`



#### `wsrep_provider (replication)`

Wsrep provider plugin

**Kind:** `plugin`

**Type:** replication

**Version:** `1.0`

**Status:** active

**Type version:** `2.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Authentication version:** `1.0.1`

**Author:** `Codership Oy`



#### `WSREP_STATUS (information schema)`

Group view information

**Kind:** `plugin`

**Type:** information schema

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** on

**Maturity:** stable

**Library:** `wsrep_info.so`

**Library version:** `1.15`

**Authentication version:** `1.0`

**Author:** `Nirbhay Choubey`



#### `xmltype (data type)`

Data type XMLTYPE

**Kind:** `plugin`

**Type:** data type

**Version:** `1.0`

**Status:** active

**Type version:** `120302.0`

**License:** GPL

**Load option:** required

**Maturity:** stable

**Authentication version:** `1.0`

**Author:** `MariaDB Corporation`



#### `test.analytics_tools`

Analytics package

**Kind:** `package`

**Security:** invoker

**Definer:** `root@localhost`

**SQL mode:** `STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,NO_AUTO_CREATE_USER,NO_ENGINE_SUBSTITUTION`

**Body:** yes

```sql
CREATE DEFINER=`root`@`localhost` PACKAGE `analytics_tools`    SQL SECURITY INVOKER
    COMMENT 'Analytics package'
 PROCEDURE refresh_cache(IN tenant BIGINT);
FUNCTION normalize(value VARCHAR(255)) RETURNS VARCHAR(255);
END

CREATE DEFINER=`root`@`localhost` PACKAGE BODY `analytics_tools` PROCEDURE refresh_cache(IN tenant BIGINT)
BEGIN
    SELECT tenant;
END;
FUNCTION normalize(value VARCHAR(255)) RETURNS VARCHAR(255)
RETURN lower(value);
END
```


#### `analytics_reader@`

**Kind:** `role`

**Host:** ``

**TLS:** none

**Privilege:** `EXECUTE` on `function test.normalize_email`

**Privilege:** `EXECUTE` on `package test.analytics_tools`

**Privilege:** `SELECT` on `schema test`

**Privilege:** `SHOW CREATE ROUTINE` on `schema test`



#### `analytics_service@localhost`

**Kind:** `user`

**Host:** `localhost`

**Authentication:** `caching_sha2_password`

**Password lifetime:** 90 days

**Account locked:** yes

**Default role:** `analytics_reader`

**TLS:** specified certificate properties

**TLS cipher:** `TLS_AES_256_GCM_SHA384`

**X.509 issuer:** `/CN=dbmd-ca`

**X.509 subject:** `/CN=dbmd-client`

**Queries per hour:** 17

**Concurrent connections:** 3

**Role:** `analytics_reader` with admin option

**Privilege:** `USAGE` on `global` with grant option

**Privilege:** `PROXY` on `proxy account 'proxy_target'@'localhost'` with grant option



#### `healthcheck@127.0.0.1`

**Kind:** `user`

**Host:** `127.0.0.1`

**Authentication:** `mysql_native_password`

**TLS:** none

**Privilege:** `USAGE` on `global`



#### `healthcheck@::1`

**Kind:** `user`

**Host:** `::1`

**Authentication:** `mysql_native_password`

**TLS:** none

**Privilege:** `USAGE` on `global`



#### `healthcheck@localhost`

**Kind:** `user`

**Host:** `localhost`

**Authentication:** `mysql_native_password`

**TLS:** none

**Privilege:** `USAGE` on `global`



#### `mariadb.sys@localhost`

**Kind:** `user`

**Host:** `localhost`

**Authentication:** `mysql_native_password`

**Password expired:** yes

**Account locked:** yes

**TLS:** none

**Privilege:** `USAGE` on `global`



#### `proxy_target@localhost`

**Kind:** `user`

**Host:** `localhost`

**Authentication:** `mysql_native_password`

**Account locked:** yes

**TLS:** none

**Privilege:** `USAGE` on `global`



#### `root@%`

**Kind:** `user`

**Host:** `%`

**Authentication:** `mysql_native_password`

**TLS:** none

**Privilege:** `ALTER` on `global` with grant option

**Privilege:** `ALTER ROUTINE` on `global` with grant option

**Privilege:** `BINLOG ADMIN` on `global` with grant option

**Privilege:** `BINLOG MONITOR` on `global` with grant option

**Privilege:** `BINLOG REPLAY` on `global` with grant option

**Privilege:** `CONNECTION ADMIN` on `global` with grant option

**Privilege:** `CREATE` on `global` with grant option

**Privilege:** `CREATE ROUTINE` on `global` with grant option

**Privilege:** `CREATE TABLESPACE` on `global` with grant option

**Privilege:** `CREATE TEMPORARY TABLES` on `global` with grant option

**Privilege:** `CREATE USER` on `global` with grant option

**Privilege:** `CREATE VIEW` on `global` with grant option

**Privilege:** `DELETE` on `global` with grant option

**Privilege:** `DELETE HISTORY` on `global` with grant option

**Privilege:** `DROP` on `global` with grant option

**Privilege:** `EVENT` on `global` with grant option

**Privilege:** `EXECUTE` on `global` with grant option

**Privilege:** `FEDERATED ADMIN` on `global` with grant option

**Privilege:** `FILE` on `global` with grant option

**Privilege:** `INDEX` on `global` with grant option

**Privilege:** `INSERT` on `global` with grant option

**Privilege:** `LOCK TABLES` on `global` with grant option

**Privilege:** `PROCESS` on `global` with grant option

**Privilege:** `READ_ONLY ADMIN` on `global` with grant option

**Privilege:** `REFERENCES` on `global` with grant option

**Privilege:** `RELOAD` on `global` with grant option

**Privilege:** `REPLICATION MASTER ADMIN` on `global` with grant option

**Privilege:** `REPLICATION SLAVE` on `global` with grant option

**Privilege:** `REPLICATION SLAVE ADMIN` on `global` with grant option

**Privilege:** `SELECT` on `global` with grant option

**Privilege:** `SET USER` on `global` with grant option

**Privilege:** `SHOW CREATE ROUTINE` on `global` with grant option

**Privilege:** `SHOW DATABASES` on `global` with grant option

**Privilege:** `SHOW VIEW` on `global` with grant option

**Privilege:** `SHUTDOWN` on `global` with grant option

**Privilege:** `SLAVE MONITOR` on `global` with grant option

**Privilege:** `SUPER` on `global` with grant option

**Privilege:** `TRIGGER` on `global` with grant option

**Privilege:** `UPDATE` on `global` with grant option

**Privilege:** `PROXY` on `proxy account ''@'%'` with grant option



#### `root@localhost`

**Kind:** `user`

**Host:** `localhost`

**Authentication:** `mysql_native_password`

**TLS:** none

**Role:** `analytics_reader` with admin option

**Privilege:** `ALTER` on `global` with grant option

**Privilege:** `ALTER ROUTINE` on `global` with grant option

**Privilege:** `BINLOG ADMIN` on `global` with grant option

**Privilege:** `BINLOG MONITOR` on `global` with grant option

**Privilege:** `BINLOG REPLAY` on `global` with grant option

**Privilege:** `CONNECTION ADMIN` on `global` with grant option

**Privilege:** `CREATE` on `global` with grant option

**Privilege:** `CREATE ROUTINE` on `global` with grant option

**Privilege:** `CREATE TABLESPACE` on `global` with grant option

**Privilege:** `CREATE TEMPORARY TABLES` on `global` with grant option

**Privilege:** `CREATE USER` on `global` with grant option

**Privilege:** `CREATE VIEW` on `global` with grant option

**Privilege:** `DELETE` on `global` with grant option

**Privilege:** `DELETE HISTORY` on `global` with grant option

**Privilege:** `DROP` on `global` with grant option

**Privilege:** `EVENT` on `global` with grant option

**Privilege:** `EXECUTE` on `global` with grant option

**Privilege:** `FEDERATED ADMIN` on `global` with grant option

**Privilege:** `FILE` on `global` with grant option

**Privilege:** `INDEX` on `global` with grant option

**Privilege:** `INSERT` on `global` with grant option

**Privilege:** `LOCK TABLES` on `global` with grant option

**Privilege:** `PROCESS` on `global` with grant option

**Privilege:** `READ_ONLY ADMIN` on `global` with grant option

**Privilege:** `REFERENCES` on `global` with grant option

**Privilege:** `RELOAD` on `global` with grant option

**Privilege:** `REPLICATION MASTER ADMIN` on `global` with grant option

**Privilege:** `REPLICATION SLAVE` on `global` with grant option

**Privilege:** `REPLICATION SLAVE ADMIN` on `global` with grant option

**Privilege:** `SELECT` on `global` with grant option

**Privilege:** `SET USER` on `global` with grant option

**Privilege:** `SHOW CREATE ROUTINE` on `global` with grant option

**Privilege:** `SHOW DATABASES` on `global` with grant option

**Privilege:** `SHOW VIEW` on `global` with grant option

**Privilege:** `SHUTDOWN` on `global` with grant option

**Privilege:** `SLAVE MONITOR` on `global` with grant option

**Privilege:** `SUPER` on `global` with grant option

**Privilege:** `TRIGGER` on `global` with grant option

**Privilege:** `UPDATE` on `global` with grant option

**Privilege:** `PROXY` on `proxy account ''@''` with grant option



## Source: `duckdb_analytics` — `DuckDB analytics`

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



## Source: `clickhouse_events` — `ClickHouse events`

Backend: `clickhouse`

### Databases

| Name | Comment |
|---|---|
| `analytics` | Engine `Atomic`; UUID `10000000-0000-0000-0000-000000000001`; Analytical application data |


### Tables

#### `analytics.country_names`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `country_id` | `UInt64` | no | - | precision 64 base 2; scale 0 |
| `parent_id` | `UInt64` | no | - | precision 64 base 2; scale 0 |
| `country_name` | `String` | no | - |  |
| `normalized_name` | `String` | no | - |  |


##### ClickHouse

**Kind:** `dictionary`

**UUID:** `20000000-0000-0000-0000-000000000006`

**Loads after:** `analytics.country_source`

**Dictionary layout:** `HASHED`

**Dictionary keys:** `country_id UInt64 IS_OBJECT_ID`

**Dictionary attributes:** `parent_id UInt64 DEFAULT 0 HIERARCHICAL, country_name String DEFAULT 'unknown' INJECTIVE, normalized_name String EXPRESSION lowerUTF8(country_name)`

**Dictionary source:** ``

**Dictionary lifetime:** 30..60 seconds

**Dictionary setting:** `max_threads_for_updates` = `4`

```sql
CREATE DICTIONARY analytics.country_names (`country_id` UInt64 IS_OBJECT_ID, `parent_id` UInt64 DEFAULT 0 HIERARCHICAL, `country_name` String DEFAULT 'unknown' INJECTIVE, `normalized_name` String EXPRESSION lowerUTF8(country_name)) PRIMARY KEY country_id SOURCE(CLICKHOUSE(HOST 'localhost' PORT 9000 USER 'default' PASSWORD '[HIDDEN]' DB 'analytics' TABLE 'country_source')) LIFETIME(MIN 30 MAX 60) LAYOUT(HASHED()) SETTINGS(max_threads_for_updates = 4)
```


#### `analytics.country_rates`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `country_id` | `UInt64` | no | - | precision 64 base 2; scale 0 |
| `valid_from` | `Date` | no | - | datetime precision 0 |
| `valid_to` | `Date` | no | - | datetime precision 0 |
| `rate` | `Float64` | no | - |  |


##### ClickHouse

**Kind:** `dictionary`

**UUID:** `20000000-0000-0000-0000-000000000016`

**Loads after:** `analytics.country_source`

**Dictionary layout:** `RANGE_HASHED`

**Dictionary keys:** `country_id UInt64`

**Dictionary attributes:** `rate Float64 DEFAULT 0`

**Dictionary source:** ``

**Dictionary lifetime:** 0..0 seconds

**Dictionary range:** MIN `valid_from` MAX `valid_to`

```sql
CREATE DICTIONARY analytics.country_rates (`country_id` UInt64, `valid_from` Date, `valid_to` Date, `rate` Float64 DEFAULT 0) PRIMARY KEY country_id SOURCE(CLICKHOUSE(HOST 'localhost' PORT 9000 USER 'default' PASSWORD '[HIDDEN]' DB 'analytics' TABLE 'country_source')) LIFETIME(MIN 0 MAX 0) LAYOUT(RANGE_HASHED()) RANGE(MIN valid_from MAX valid_to)
```


#### `analytics.country_source`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `country_id` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0; keys `primary, sorting` |
| `parent_id` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0 |
| `country_name` | `String` | no | - | serialization `Default`; statistics `Uniq(auto)` |
| `valid_from` | `Date` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; datetime precision 0 |
| `valid_to` | `Date` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; datetime precision 0 |
| `rate` | `Float64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)` |


##### ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000005`

**Engine:** `MergeTree ORDER BY country_id SETTINGS index_granularity = 8192`

**Primary key:** `country_id`

**Sorting key:** `country_id`

**Storage policy:** `default`

**Setting:** `index_granularity` = `8192`

**Loads before:** `analytics.country_names`

**Loads before:** `analytics.country_rates`

```sql
CREATE TABLE analytics.country_source (`country_id` UInt64, `parent_id` UInt64, `country_name` String, `valid_from` Date, `valid_to` Date, `rate` Float64) ENGINE = MergeTree ORDER BY country_id SETTINGS index_granularity = 8192
```


#### `analytics.event_counts`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `event_type` | `LowCardinality(String)` | no | - | statistics `Uniq(auto)`; keys `primary, sorting` |
| `total` | `AggregateFunction(count)` | no | - |  |


##### ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000002`

**Engine:** `AggregatingMergeTree ORDER BY event_type SETTINGS index_granularity = 8192`

**Primary key:** `event_type`

**Sorting key:** `event_type`

**Storage policy:** `default`

**Setting:** `index_granularity` = `8192`

```sql
CREATE TABLE analytics.event_counts (`event_type` LowCardinality(String), `total` AggregateFunction(count)) ENGINE = AggregatingMergeTree ORDER BY event_type SETTINGS index_granularity = 8192
```


#### `analytics.events`

Immutable analytical events

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `UInt32` | no | - | Owning tenant; serialization `Default`; statistics `minmax(auto)`; precision 32 base 2; scale 0; keys `primary, sorting` |
| `event_id` | `UUID` | no | - | serialization `Default`; keys `primary, sorting` |
| `occurred_at` | `DateTime64(3, 'UTC')` | no | - | codec `CODEC(DoubleDelta, ZSTD(1))`; serialization `Default`; statistics `minmax(auto)`; datetime precision 3; keys `partition, sorting` |
| `event_type` | `LowCardinality(String)` | no | `'unknown'` | `default` expression |
| `payload` | `String` | no | - | codec `CODEC(ZSTD(3))`; serialization `Default` |
| `vector` | `QBit(Float32, 8)` | no | - |  |
| `expires_at` | `DateTime` | no | `toDateTime(occurred_at) + toIntervalDay(30)` | serialization `Default`; statistics `minmax(auto)`; datetime precision 0; `materialized` expression |
| `version` | `UInt64` | no | - | serialization `Default`; statistics `minmax(auto)`; precision 64 base 2; scale 0 |
| `deleted` | `UInt8` | no | `0` | serialization `Default`; statistics `minmax(auto)`; precision 8 base 2; scale 0; `default` expression |


##### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `positive_tenant` | `assumption` | - | `tenant_id > 0` |
| `valid_deleted` | `check` | - | `deleted IN (0, 1)` |


##### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `auto_minmax_index_expires_at` | `expires_at` | no | clickhouse `minmax()` granularity 1; implicit | - |
| `auto_minmax_index_occurred_at` | `occurred_at` | no | clickhouse `minmax()` granularity 1; implicit | - |
| `payload_text` | `payload` | no | clickhouse `text(tokenizer = 'splitByNonAlpha')` granularity 100000000 | - |
| `payload_tokens` | `lower(payload)` | no | clickhouse `tokenbf_v1(1024, 3, 0)` granularity 4 | - |


##### ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000001`

**Engine:** `ReplacingMergeTree(version, deleted) PARTITION BY toYYYYMM(occurred_at) PRIMARY KEY (tenant_id, event_id) ORDER BY (tenant_id, event_id, occurred_at) TTL toDateTime(occurred_at) + toIntervalDay(365) SETTINGS index_granularity = 4096, deduplicate_merge_projection_mode = 'drop', auto_statistics_types = 'minmax', add_minmax_index_for_temporal_columns = 1`

**Engine argument:** 1: `version`

**Engine argument:** 2: `deleted`

**Partition key:** `toYYYYMM(occurred_at)`

**Primary key:** `tenant_id, event_id`

**Sorting key:** `tenant_id, event_id, occurred_at`

**Storage policy:** `default`

**TTL:** `toDateTime(occurred_at) + toIntervalDay(365)`; `delete`

**Setting:** `add_minmax_index_for_temporal_columns` = `1`

**Setting:** `auto_statistics_types` = `'minmax'`

**Setting:** `deduplicate_merge_projection_mode` = `'drop'`

**Setting:** `index_granularity` = `4096`

**Depends on:** `analytics.event_counts_mv`

**Projection:** `by_event_type` `Aggregate sorted by event_type`: `SELECT event_type, count() GROUP BY event_type`

**Projection:** `by_time` `index occurred_at type basic`: `occurred_at`

```sql
CREATE TABLE analytics.events (`tenant_id` UInt32 COMMENT 'Owning tenant', `event_id` UUID, `occurred_at` DateTime64(3, 'UTC') CODEC(DoubleDelta, ZSTD(1)), `event_type` LowCardinality(String) DEFAULT 'unknown', `payload` String CODEC(ZSTD(3)), `vector` QBit(Float32, 8), `expires_at` DateTime MATERIALIZED toDateTime(occurred_at) + toIntervalDay(30), `version` UInt64, `deleted` UInt8 DEFAULT 0, INDEX payload_tokens lower(payload) TYPE tokenbf_v1(1024, 3, 0) GRANULARITY 4, INDEX payload_text payload TYPE text(tokenizer = 'splitByNonAlpha') GRANULARITY 100000000, CONSTRAINT valid_deleted CHECK deleted IN (0, 1), CONSTRAINT positive_tenant ASSUME tenant_id > 0, PROJECTION by_event_type (SELECT event_type, count() GROUP BY event_type), PROJECTION by_time INDEX occurred_at TYPE basic) ENGINE = ReplacingMergeTree(version, deleted) PARTITION BY toYYYYMM(occurred_at) PRIMARY KEY (tenant_id, event_id) ORDER BY (tenant_id, event_id, occurred_at) TTL toDateTime(occurred_at) + toIntervalDay(365) SETTINGS index_granularity = 4096, deduplicate_merge_projection_mode = 'drop', auto_statistics_types = 'minmax', add_minmax_index_for_temporal_columns = 1 COMMENT 'Immutable analytical events'
```


#### `analytics.modern_storage`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0; keys `primary, sorting` |
| `measurement` | `Float64` | no | - | codec `CODEC(ALP, ZSTD(1))`; serialization `Default`; statistics `Uniq(auto),minmax(auto)` |
| `attributes` | `Map(String, String)` | no | - |  |


##### ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000014`

**Engine:** `CoalescingMergeTree ORDER BY id SETTINGS map_serialization_version = 'with_buckets', map_serialization_version_for_zero_level_parts = 'basic', map_buckets_strategy = 'linear', map_buckets_coefficient = 0.5, map_buckets_min_avg_size = 0, max_buckets_in_map = 64, index_granularity = 8192`

**Primary key:** `id`

**Sorting key:** `id`

**Storage policy:** `default`

**Setting:** `index_granularity` = `8192`

**Setting:** `map_buckets_coefficient` = `0.5`

**Setting:** `map_buckets_min_avg_size` = `0`

**Setting:** `map_buckets_strategy` = `'linear'`

**Setting:** `map_serialization_version` = `'with_buckets'`

**Setting:** `map_serialization_version_for_zero_level_parts` = `'basic'`

**Setting:** `max_buckets_in_map` = `64`

```sql
CREATE TABLE analytics.modern_storage (`id` UInt64, `measurement` Float64 CODEC(ALP, ZSTD(1)), `attributes` Map(String, String)) ENGINE = CoalescingMergeTree ORDER BY id SETTINGS map_serialization_version = 'with_buckets', map_serialization_version_for_zero_level_parts = 'basic', map_buckets_strategy = 'linear', map_buckets_coefficient = 0.5, map_buckets_min_avg_size = 0, max_buckets_in_map = 64, index_granularity = 8192
```


#### `analytics.refresh_rollups`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `UInt32` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 32 base 2; scale 0; keys `primary, sorting` |
| `total` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0 |


##### ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000011`

**Engine:** `MergeTree ORDER BY tenant_id SETTINGS index_granularity = 8192`

**Primary key:** `tenant_id`

**Sorting key:** `tenant_id`

**Storage policy:** `default`

**Setting:** `index_granularity` = `8192`

```sql
CREATE TABLE analytics.refresh_rollups (`tenant_id` UInt32, `total` UInt64) ENGINE = MergeTree ORDER BY tenant_id SETTINGS index_granularity = 8192
```


#### `analytics.refresh_snapshots`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `UInt32` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 32 base 2; scale 0; keys `primary, sorting` |
| `captured_at` | `DateTime` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; datetime precision 0 |


##### ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000010`

**Engine:** `MergeTree ORDER BY tenant_id SETTINGS index_granularity = 8192`

**Primary key:** `tenant_id`

**Sorting key:** `tenant_id`

**Storage policy:** `default`

**Setting:** `index_granularity` = `8192`

```sql
CREATE TABLE analytics.refresh_snapshots (`tenant_id` UInt32, `captured_at` DateTime) ENGINE = MergeTree ORDER BY tenant_id SETTINGS index_granularity = 8192
```


#### `analytics.remote_accounts`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `account_id` | `UInt64` | no | - | precision 64 base 2; scale 0 |
| `email` | `String` | no | - |  |


##### ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000007`

**Engine:** `MySQL('127.0.0.1:3306', 'remote_app', 'accounts', 'remote_reader', '[HIDDEN]')`

**Engine argument:** 1: `'127.0.0.1:3306'`

**Engine argument:** 2: `'remote_app'`

**Engine argument:** 3: `'accounts'`

**Engine argument:** 4: `'remote_reader'`

**Engine argument:** 5: `'[HIDDEN]'`

```sql
CREATE TABLE analytics.remote_accounts (`account_id` UInt64, `email` String) ENGINE = MySQL('127.0.0.1:3306', 'remote_app', 'accounts', 'remote_reader', '[HIDDEN]')
```


#### `analytics.retention_matrix`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `event_id` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0; keys `primary, sorting` |
| `occurred_at` | `DateTime` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; datetime precision 0 |
| `expires_at` | `DateTime` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; TTL `expires_at + toIntervalDay(1)`; datetime precision 0 |
| `deleted` | `UInt8` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 8 base 2; scale 0 |
| `payload` | `String` | no | - | serialization `Default`; statistics `Uniq(auto)` |


##### ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000008`

**Engine:** `MergeTree ORDER BY event_id TTL occurred_at + toIntervalDay(7) TO DISK 'default', occurred_at + toIntervalDay(10) TO VOLUME 'default', occurred_at + toIntervalDay(14) RECOMPRESS CODEC(ZSTD(9)), occurred_at + toIntervalDay(30) WHERE deleted = 1 SETTINGS index_granularity = 4096`

**Primary key:** `event_id`

**Sorting key:** `event_id`

**Storage policy:** `default`

**TTL:** `occurred_at + toIntervalDay(7)`; `move to disk 'default'`

**TTL:** `occurred_at + toIntervalDay(10)`; `move to volume 'default'`

**TTL:** `occurred_at + toIntervalDay(14)`; `recompress CODEC(ZSTD(9))`

**TTL:** `occurred_at + toIntervalDay(30)`; `delete where deleted = 1`

**Setting:** `index_granularity` = `4096`

```sql
CREATE TABLE analytics.retention_matrix (`event_id` UInt64, `occurred_at` DateTime, `expires_at` DateTime TTL expires_at + toIntervalDay(1), `deleted` UInt8, `payload` String) ENGINE = MergeTree ORDER BY event_id TTL occurred_at + toIntervalDay(7) TO DISK 'default', occurred_at + toIntervalDay(10) TO VOLUME 'default', occurred_at + toIntervalDay(14) RECOMPRESS CODEC(ZSTD(9)), occurred_at + toIntervalDay(30) WHERE deleted = 1 SETTINGS index_granularity = 4096
```


#### `analytics.retention_rollup`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0; keys `primary, sorting` |
| `occurred_at` | `DateTime` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; datetime precision 0 |
| `amount` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0 |


##### ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000009`

**Engine:** `MergeTree ORDER BY tenant_id TTL occurred_at + toIntervalDay(30) GROUP BY tenant_id SET amount = sum(amount) SETTINGS index_granularity = 8192`

**Primary key:** `tenant_id`

**Sorting key:** `tenant_id`

**Storage policy:** `default`

**TTL:** `occurred_at + toIntervalDay(30)`; `group by tenant_id set amount = sum(amount)`

**Setting:** `index_granularity` = `8192`

```sql
CREATE TABLE analytics.retention_rollup (`tenant_id` UInt64, `occurred_at` DateTime, `amount` UInt64) ENGINE = MergeTree ORDER BY tenant_id TTL occurred_at + toIntervalDay(30) GROUP BY tenant_id SET amount = sum(amount) SETTINGS index_granularity = 8192
```


#### `analytics.s3_archive`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `payload` | `String` | no | - |  |


##### ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000015`

**Engine:** `S3('s3://dbmd-audit/archive.parquet', 'Parquet', storage_class_name = 'INTELLIGENT_TIERING')`

**Engine argument:** 1: `'s3://dbmd-audit/archive.parquet'`

**Engine argument:** 2: `'Parquet'`

**Engine parameter:** `storage_class_name` = `'INTELLIGENT_TIERING'`

```sql
CREATE TABLE analytics.s3_archive (`payload` String) ENGINE = S3('s3://dbmd-audit/archive.parquet', 'Parquet', storage_class_name = 'INTELLIGENT_TIERING')
```


#### `analytics.window_event_counts`

##### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `total` | `UInt64` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; precision 64 base 2; scale 0 |
| `window_end` | `DateTime` | no | - | serialization `Default`; statistics `Uniq(auto),minmax(auto)`; datetime precision 0; keys `primary, sorting` |


##### ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000018`

**Engine:** `MergeTree ORDER BY window_end SETTINGS index_granularity = 8192`

**Primary key:** `window_end`

**Sorting key:** `window_end`

**Storage policy:** `default`

**Setting:** `index_granularity` = `8192`

```sql
CREATE TABLE analytics.window_event_counts (`total` UInt64, `window_end` DateTime) ENGINE = MergeTree ORDER BY window_end SETTINGS index_granularity = 8192
```


### Views

#### `analytics.active_events`

Non-deleted events

**Kind:** `view`

**UUID:** `20000000-0000-0000-0000-000000000004`

**Definer:** `default`

**SQL security:** `invoker`

**AS SELECT:** `SELECT tenant_id, event_id, occurred_at, event_type FROM analytics.events WHERE deleted = 0`

| Column | Type | Nullable |
|---|---|---|
| `tenant_id` | `UInt32` | no |
| `event_id` | `UUID` | no |
| `occurred_at` | `DateTime64(3, 'UTC')` | no |
| `event_type` | `LowCardinality(String)` | no |


```sql
CREATE VIEW analytics.active_events (`tenant_id` UInt32, `event_id` UUID, `occurred_at` DateTime64(3, 'UTC'), `event_type` LowCardinality(String)) DEFINER = default SQL SECURITY INVOKER COMMENT 'Non-deleted events' AS SELECT tenant_id, event_id, occurred_at, event_type FROM analytics.events WHERE deleted = 0
```

#### `analytics.event_counts_mv`

**Kind:** `materialized_view`

**UUID:** `20000000-0000-0000-0000-000000000003`

**Target:** `analytics.event_counts`

**AS SELECT:** `SELECT event_type, countState() AS total FROM analytics.events GROUP BY event_type`

| Column | Type | Nullable |
|---|---|---|
| `event_type` | `LowCardinality(String)` | no |
| `total` | `AggregateFunction(count)` | no |


```sql
CREATE MATERIALIZED VIEW analytics.event_counts_mv TO analytics.event_counts (`event_type` LowCardinality(String), `total` AggregateFunction(count)) AS SELECT event_type, countState() AS total FROM analytics.events GROUP BY event_type
```

#### `analytics.events_by_tenant`

**Kind:** `view`

**UUID:** `20000000-0000-0000-0000-000000000017`

**AS SELECT:** `SELECT tenant_id, event_id, occurred_at, event_type FROM analytics.events WHERE tenant_id = {requested_tenant:UInt32}`

**Parameter:** `requested_tenant` `UInt32`

| Column | Type | Nullable |
|---|---|---|


```sql
CREATE VIEW analytics.events_by_tenant AS SELECT tenant_id, event_id, occurred_at, event_type FROM analytics.events WHERE tenant_id = {requested_tenant:UInt32}
```

#### `analytics.refresh_base`

**Kind:** `materialized_view`

**UUID:** `20000000-0000-0000-0000-000000000012`

**Target:** `analytics.refresh_snapshots`

**Refresh:** `every 1 HOUR`

**Refresh offset:** `5 MINUTE`

**Refresh randomization:** `1 MINUTE`

**Refresh mode:** `append`

**Refresh setting:** `refresh_retries` = `5`

**Definer:** `default`

**SQL security:** `definer`

**AS SELECT:** `SELECT tenant_id, now() AS captured_at FROM analytics.events`

| Column | Type | Nullable |
|---|---|---|
| `tenant_id` | `UInt32` | no |
| `captured_at` | `DateTime` | no |


```sql
CREATE MATERIALIZED VIEW analytics.refresh_base REFRESH EVERY 1 HOUR OFFSET 5 MINUTE RANDOMIZE FOR 1 MINUTE SETTINGS refresh_retries = 5 APPEND TO analytics.refresh_snapshots (`tenant_id` UInt32, `captured_at` DateTime) DEFINER = default SQL SECURITY DEFINER AS SELECT tenant_id, now() AS captured_at FROM analytics.events
```

#### `analytics.refresh_dependent`

**Kind:** `materialized_view`

**UUID:** `20000000-0000-0000-0000-000000000013`

**Target:** `analytics.refresh_rollups`

**Refresh:** `after 2 HOUR`

**Refresh mode:** `replace`

**Refresh depends on:** `analytics.refresh_base`

**Refresh setting:** `refresh_retries` = `3`

**Definer:** `default`

**SQL security:** `definer`

**AS SELECT:** `SELECT tenant_id, count() AS total FROM analytics.events GROUP BY tenant_id`

| Column | Type | Nullable |
|---|---|---|
| `tenant_id` | `UInt32` | no |
| `total` | `UInt64` | no |


```sql
CREATE MATERIALIZED VIEW analytics.refresh_dependent REFRESH AFTER 2 HOUR DEPENDS ON analytics.refresh_base SETTINGS refresh_retries = 3 TO analytics.refresh_rollups (`tenant_id` UInt32, `total` UInt64) DEFINER = default SQL SECURITY DEFINER AS SELECT tenant_id, count() AS total FROM analytics.events GROUP BY tenant_id
```

#### `analytics.windowed_events`

**Kind:** `window_view`

**UUID:** `20000000-0000-0000-0000-000000000019`

**Target:** `analytics.window_event_counts`

**Window inner engine:** `AggregatingMergeTree ORDER BY tuple()`

**Watermark:** `STRICTLY_ASCENDING`

**AS SELECT:** `SELECT count(event_id) AS total, tumbleEnd(window_id) AS window_end FROM analytics.retention_matrix GROUP BY tumble(occurred_at, toIntervalSecond('5')) AS window_id`

| Column | Type | Nullable |
|---|---|---|
| `total` | `UInt64` | no |
| `window_end` | `DateTime` | no |


```sql
CREATE WINDOW VIEW analytics.windowed_events TO analytics.window_event_counts (`total` UInt64, `window_end` DateTime) INNER ENGINE = AggregatingMergeTree ORDER BY tuple() WATERMARK STRICTLY_ASCENDING AS SELECT count(event_id) AS total, tumbleEnd(window_id) AS window_end FROM analytics.retention_matrix GROUP BY tumble(occurred_at, toIntervalSecond('5')) AS window_id
```

#### `analytics.windowed_events_owned`

**Kind:** `window_view`

**UUID:** `20000000-0000-0000-0000-000000000020`

**Window inner engine:** `AggregatingMergeTree ORDER BY tuple()`

**Window storage engine:** `MergeTree ORDER BY window_end SETTINGS index_granularity = 8192`

**Watermark:** `ASCENDING`

**Allowed lateness:** `toIntervalSecond('2')`

**AS SELECT:** `SELECT count(event_id) AS total, tumbleEnd(window_id) AS window_end FROM analytics.retention_matrix GROUP BY tumble(occurred_at, toIntervalSecond('5')) AS window_id`

| Column | Type | Nullable |
|---|---|---|
| `total` | `UInt64` | no |
| `window_end` | `DateTime` | no |


```sql
CREATE WINDOW VIEW analytics.windowed_events_owned (`total` UInt64, `window_end` DateTime) INNER ENGINE = AggregatingMergeTree ORDER BY tuple() ENGINE = MergeTree ORDER BY window_end SETTINGS index_granularity = 8192 WATERMARK ASCENDING ALLOWED_LATENESS toIntervalSecond('2') AS SELECT count(event_id) AS total, tumbleEnd(window_id) AS window_end FROM analytics.retention_matrix GROUP BY tumble(occurred_at, toIntervalSecond('5')) AS window_id
```

### Functions

#### `analytics_normalize`

**Kind:** `user_defined_function`

**Origin:** `SQL-defined`

```sql
CREATE FUNCTION analytics_normalize AS value -> lowerUTF8(value)
```


### Access and workload objects

#### `analytics_service`

**Kind:** `user`

**Storage:** `local_directory`

**Authentication:** `sha256_password`

**Hosts:** `name localhost`

**Default roles:** `analytics_reader`

**Grantees:** `all`

**Default database:** `analytics`

**Role grant:** `analytics_reader default with admin option`



#### `analytics_reader`

**Kind:** `role`

**Storage:** `local_directory`

**Privilege:** `SELECT on analytics with grant option`



#### `tenant_events ON analytics.events`

**Kind:** `row_policy`

**Database:** `analytics`

**Table:** `events`

**Mode:** `permissive`

**Applies to:** `analytics_reader`

**Storage:** `local_directory`

**SELECT filter:** `tenant_id > 0`



#### `analytics_quota`

**Kind:** `quota`

**Storage:** `local_directory`

**Keys:** `user_name`

**Applies to:** `analytics_reader`

**Limit:** `3600 seconds, queries=100, query_selects=90, query_inserts=10, errors=5, result_rows=1000, result_bytes=2000, read_rows=3000, read_bytes=4000, written_bytes=5000, failed_sequential_authentications=3, queries_per_normalized_hash=7, execution_time=60`



#### `analytics_profile`

**Kind:** `settings_profile`

**Storage:** `local_directory`

**Applies to:** `analytics_reader`

**Element:** `max_threads = 4 min 1 max 8 writable`



#### `analytics_remote`

**Kind:** `named_collection`

**Source:** `SQL`

**Entry:** `host`; overridable

**Entry:** `password`; not overridable

```sql
CREATE NAMED COLLECTION analytics_remote AS host = '[HIDDEN]' OVERRIDABLE, password = '[HIDDEN]' NOT OVERRIDABLE
```


#### `analytics_cpu`

**Kind:** `resource`

**Unit:** `CPUNanosecond`

**Operation:** `master thread`

**Operation:** `worker thread`

```sql
CREATE RESOURCE analytics_cpu (MASTER THREAD, WORKER THREAD)
```


#### `analytics_all`

**Kind:** `workload`

```sql
CREATE WORKLOAD analytics_all
```


#### `analytics_interactive`

**Kind:** `workload`

**Parent:** `analytics_all`

**Setting:** `max_concurrent_threads` = `8` for `analytics_cpu`

```sql
CREATE WORKLOAD analytics_interactive IN analytics_all SETTINGS max_concurrent_threads = 8 FOR analytics_cpu
```


