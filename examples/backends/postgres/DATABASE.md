# Database: `PostgreSQL application catalog`

## Database

### `dbmd`

**Owner:** `dbmd`

**Encoding:** `UTF8`

**Locale provider:** `libc`

**LC_COLLATE:** `en_US.utf8`

**LC_CTYPE:** `en_US.utf8`

**Tablespace:** `pg_default`

**Template:** no

**Allows connections:** yes

**Connection limit:** -1



## Namespaces

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


## Enum Types

| Type | Owner | Values | Comment |
|---|---|---|---|
| `catalog.account_state` | `dbmd` | `active, suspended` | - |
| `infrastructure.label_a` | `dbmd` | `a, b` | - |
| `infrastructure.label_b` | `dbmd` | `a, b` | - |
| `secure.event_state` | `dbmd_acl_owner` | `pending, complete` | - |


## Composite Types

### `storage.device_row`

**Owner:** `dbmd`

**Attribute:** `device_id` `bigint`

**Attribute:** `payload` `text`; collation `pg_catalog."default"`

```sql
CREATE TYPE "storage"."device_row" AS (
    "device_id" bigint,
    "payload" text COLLATE pg_catalog."default"
);
```


## Domains

### `secure.event_code`

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


## Base and Shell Types

### `infrastructure.label_c`

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


### `type_system.pending_value`

Forward-declared shell type

**Kind:** `shell`

**Owner:** `dbmd`

```sql
CREATE TYPE "type_system"."pending_value";
```


### `type_system.scalar_token`

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


## Range and Multirange Types

### `type_system.measurement_range`

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


## Sequences

### `audit.accounts_id_seq`

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


### `automation.invoice_number`

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


### `catalog.accounts_id_seq`

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


### `secure.event_sequence`

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


## Tables

### `advanced.deleted_orders`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | yes | - | storage `plain` |
| `deleted_at` | `timestamp with time zone` | no | `CURRENT_TIMESTAMP` | storage `plain` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `deleted_orders_deleted_at_not_null` | `not_null` | `deleted_at` | `NOT NULL deleted_at` |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



### `advanced.orders`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain` |
| `customer_id` | `bigint` | no | - | storage `plain` |
| `region` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |
| `amount` | `numeric(12,2)` | no | - | storage `main` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `orders_amount_not_null` | `not_null` | `amount` | `NOT NULL amount` |
| `orders_customer_id_not_null` | `not_null` | `customer_id` | `NOT NULL customer_id` |
| `orders_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `orders_pkey` | `primary_key` | `id` | `PRIMARY KEY (id)`; no inherit |
| `orders_region_not_null` | `not_null` | `region` | `NOT NULL region` |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `orders_pkey` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `orders_pkey` | - |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



### `audit.account_limits`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `account_id` | `bigint` | no | - | storage `plain` |
| `minimum_balance` | `integer` | no | - | storage `plain` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `account_limits_account_id_not_null` | `not_null` | `account_id` | `NOT NULL account_id` |
| `account_limits_minimum_balance_not_null` | `not_null` | `minimum_balance` | `NOT NULL minimum_balance` |
| `account_limits_pkey` | `primary_key` | `account_id` | `PRIMARY KEY (account_id)`; no inherit |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `account_limits_pkey` | `account_id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `account_limits_pkey` | - |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



### `audit.accounts`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | identity `always`; storage `plain` |
| `email` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |
| `balance` | `integer` | no | `0` | storage `plain` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `accounts_balance_not_null` | `not_null` | `balance` | `NOT NULL balance` |
| `accounts_email_not_null` | `not_null` | `email` | `NOT NULL email` |
| `accounts_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `accounts_pkey` | `primary_key` | `id` | `PRIMARY KEY (id)`; no inherit |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_pkey` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `accounts_pkey` | - |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



### `audit.partitioned_events`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain` |
| `occurred_on` | `date` | no | - | storage `plain` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `partitioned_events_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `partitioned_events_occurred_on_not_null` | `not_null` | `occurred_on` | `NOT NULL occurred_on` |


#### PostgreSQL

**Kind:** `partitioned_table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Partition key:** `RANGE (occurred_on)`



### `audit.partitioned_events_2026`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain`; inherited only |
| `occurred_on` | `date` | no | - | storage `plain`; inherited only |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `partitioned_events_id_not_null` | `not_null` | `id` | `NOT NULL id`; inherited |
| `partitioned_events_occurred_on_not_null` | `not_null` | `occurred_on` | `NOT NULL occurred_on`; inherited |


#### PostgreSQL

**Kind:** `partition`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Inherits:** `audit.partitioned_events`

**Partition parent:** `audit.partitioned_events`

**Partition bound:** `FOR VALUES FROM ('2026-01-01') TO ('2027-01-01')`



### `automation.invoices`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | `nextval('automation.invoice_number'::regclass)` | storage `plain` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `invoices_id_not_null` | `not_null` | `id` | `NOT NULL id` |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



### `billing.accounts`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `account_id` | `bigint` | no | - | storage `plain` |
| `email` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `accounts_account_id_not_null` | `not_null` | `account_id` | `NOT NULL account_id` |
| `accounts_pk` | `primary_key` | `tenant_id, account_id` | `PRIMARY KEY (tenant_id, account_id)`; no inherit |
| `accounts_tenant_email_unique` | `unique` | `tenant_id, email` | `UNIQUE (tenant_id, email)`; no inherit |
| `accounts_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_pk` | `tenant_id` ascending opclass `pg_catalog.int8_ops` nulls `last`, `account_id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `accounts_pk` | - |
| `accounts_tenant_email_unique` | `tenant_id` ascending opclass `pg_catalog.int8_ops` nulls `last`, `email` ascending collate `pg_catalog."default"` opclass `pg_catalog.text_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `accounts_tenant_email_unique` | - |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



### `billing.invoices`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `account_id` | `bigint` | no | - | storage `plain` |
| `invoice_number` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `invoices_account_fk` | `foreign_key` | `tenant_id, account_id` | `FOREIGN KEY (tenant_id, account_id) REFERENCES billing.accounts(tenant_id, account_id) MATCH FULL ON UPDATE CASCADE ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED`; no inherit |
| `invoices_account_id_not_null` | `not_null` | `account_id` | `NOT NULL account_id` |
| `invoices_invoice_number_not_null` | `not_null` | `invoice_number` | `NOT NULL invoice_number` |
| `invoices_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



### `catalog.accounts`

Application accounts

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | identity `always`; storage `plain` |
| `email` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |
| `state` | `catalog.account_state` | no | `'active'::catalog.account_state` | enum values `active, suspended`; storage `plain` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `accounts_email_not_null` | `not_null` | `email` | `NOT NULL email` |
| `accounts_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `accounts_pkey` | `primary_key` | `id` | `PRIMARY KEY (id)`; no inherit |
| `accounts_state_not_null` | `not_null` | `state` | `NOT NULL state` |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_pkey` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `accounts_pkey` | - |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



### `search.documents`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain` |
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `title` | `text` | no | - | collation `pg_catalog."C"`; storage `extended` |
| `body` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |
| `published` | `boolean` | no | `false` | storage `plain` |
| `active_window` | `int4range` | yes | - | storage `extended` |


#### Constraints

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


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `documents_active_window_exclude` | `active_window` ascending opclass `pg_catalog.range_ops` | no | postgres `gist`; owner `dbmd`; constraint `documents_active_window_exclude` | - |
| `documents_brin_idx` | `id` ascending opclass `pg_catalog.int8_bloom_ops` parameters `n_distinct_per_range=32, false_positive_rate=0.05` | no | postgres `brin`; owner `dbmd` | - |
| `documents_cluster_idx` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | no | postgres `btree`; clustered; owner `dbmd` | - |
| `documents_lookup_idx` | `tenant_id` ascending opclass `pg_catalog.int8_ops` nulls `last`, `lower(title)` descending collate `pg_catalog."C"` opclass `pg_catalog.text_ops` nulls `first` | yes | postgres `btree`; include `body`; nulls not distinct; owner `dbmd`; option `fillfactor=75`; Published-document lookup | `published` |
| `documents_replica_idx` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; replica identity; owner `dbmd` | - |
| `documents_title_unique` | `tenant_id` ascending opclass `pg_catalog.int8_ops` nulls `last`, `title` ascending collate `pg_catalog."C"` opclass `pg_catalog.text_ops` nulls `last` | yes | postgres `btree`; nulls not distinct; owner `dbmd`; constraint `documents_title_unique` | - |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `index`

**Access method:** `heap`



### `secure.events`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | no | - | storage `plain` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `events_id_not_null` | `not_null` | `id` | `NOT NULL id` |
| `events_pkey` | `primary_key` | `id` | `PRIMARY KEY (id)`; no inherit |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `events_pkey` | `id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd_acl_owner`; constraint `events_pkey` | - |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd_acl_owner`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



### `secure.remote_events`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `bigint` | yes | - | storage `plain` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


#### PostgreSQL

**Kind:** `foreign_table`

**Owner:** `dbmd_acl_owner`

**Persistence:** `permanent`

**Replica identity:** `nothing`

**Foreign server:** `secure_server`

**Foreign-data wrapper:** `postgres_fdw`

**Foreign option:** `table_name=events`



### `storage.event_payloads`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `event_id` | `bigint` | no | - | storage `plain` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `external`; compression `lz4`; statistics target 777; option `n_distinct=-0.5` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `event_payloads_event_id_not_null` | `not_null` | `event_id` | `NOT NULL event_id` |
| `event_payloads_pkey` | `primary_key` | `event_id` | `PRIMARY KEY (event_id)`; no inherit |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `event_payloads_pkey` | `event_id` ascending opclass `pg_catalog.int8_ops` nulls `last` | yes | postgres `btree`; owner `dbmd`; constraint `event_payloads_pkey` | - |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `unlogged`

**Replica identity:** `full`

**Access method:** `heap`

**Option:** `fillfactor=70`



### `storage.remote_events`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `event_id` | `bigint` | yes | - | storage `plain`; foreign option `remote_name=external_id` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


#### PostgreSQL

**Kind:** `foreign_table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `nothing`

**Foreign server:** `fixture_server`

**Foreign-data wrapper:** `fixture_wrapper`

**Foreign option:** `schema_name=remote`

**Foreign option:** `table_name=events`



### `storage.typed_devices`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `device_id` | `bigint` | yes | - | storage `plain` |
| `payload` | `text` | yes | - | collation `pg_catalog."default"`; storage `extended` |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Of type:** `storage.device_row`



### `temporal.accounts`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `account_id` | `bigint` | yes | - | storage `plain` |
| `email` | `text` | no | - | collation `temporal.unicode_fast`; storage `extended` |
| `base_amount` | `integer` | yes | - | storage `plain` |
| `virtual_amount` | `integer` | yes | - | generated `virtual` as `base_amount * 2`; storage `plain` |
| `stored_amount` | `integer` | yes | - | generated `stored` as `base_amount * 3`; storage `plain` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `accounts_amount_nonnegative` | `check` | `base_amount` | `CHECK (base_amount >= 0) NOT ENFORCED`; not validated; not enforced |
| `accounts_email_required` | `not_null` | `email` | `NOT NULL email` |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



### `temporal.plan_assignments`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `plan_id` | `bigint` | no | - | storage `plain` |
| `valid_at` | `daterange` | no | - | storage `extended` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `assignments_plan_period` | `foreign_key` | `plan_id, valid_at` | `FOREIGN KEY (plan_id, PERIOD valid_at) REFERENCES temporal.plan_versions(plan_id, PERIOD valid_at) NOT ENFORCED`; not validated; not enforced; temporal; no inherit |
| `plan_assignments_plan_id_not_null` | `not_null` | `plan_id` | `NOT NULL plan_id` |
| `plan_assignments_valid_at_not_null` | `not_null` | `valid_at` | `NOT NULL valid_at` |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



### `temporal.plan_versions`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `plan_id` | `bigint` | no | - | storage `plain` |
| `valid_at` | `daterange` | no | - | storage `extended` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `plan_versions_identity` | `unique` | `plan_id, valid_at` | `UNIQUE (plan_id, valid_at WITHOUT OVERLAPS)`; temporal; no inherit; operators `pg_catalog.=(bigint,bigint), pg_catalog.&&(pg_catalog.anyrange,pg_catalog.anyrange)` |
| `plan_versions_plan_id_not_null` | `not_null` | `plan_id` | `NOT NULL plan_id` |
| `plan_versions_valid_at_not_null` | `not_null` | `valid_at` | `NOT NULL valid_at` |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `plan_versions_identity` | `plan_id` ascending opclass `public.gist_int8_ops`, `valid_at` ascending opclass `pg_catalog.range_ops` | yes | postgres `gist`; owner `dbmd`; constraint `assignments_plan_period` | - |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



### `tenancy.base_events`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `payload` | `jsonb` | no | - | storage `extended` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `base_events_payload_not_null` | `not_null` | `payload` | `NOT NULL payload` |
| `base_events_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Policy:** `tenant_events` `select` to `PUBLIC` (restrictive); using `tenant_id = current_setting('app.tenant_id'::text)::bigint`; Restricts events to the active tenant

Row-level security enabled.

Row-level security forced for the table owner.



### `tenancy.events`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain` |
| `created_at` | `date` | no | - | storage `plain` |
| `payload` | `jsonb` | no | - | storage `extended` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `events_created_at_not_null` | `not_null` | `created_at` | `NOT NULL created_at` |
| `events_payload_not_null` | `not_null` | `payload` | `NOT NULL payload` |
| `events_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id` |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `events_created_idx` | `created_at` ascending opclass `pg_catalog.date_ops` nulls `last` | no | postgres `btree`; owner `dbmd`; partitioned | - |


#### PostgreSQL

**Kind:** `partitioned_table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Partition key:** `RANGE (created_at)`



### `tenancy.events_2025`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain`; inherited only |
| `created_at` | `date` | no | - | storage `plain`; inherited only |
| `payload` | `jsonb` | no | - | storage `extended`; inherited only |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `events_created_at_not_null` | `not_null` | `created_at` | `NOT NULL created_at`; inherited |
| `events_payload_not_null` | `not_null` | `payload` | `NOT NULL payload`; inherited |
| `events_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id`; inherited |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `events_2025_created_idx` | `created_at` ascending opclass `pg_catalog.date_ops` nulls `last` | no | postgres `btree`; owner `dbmd`; option `fillfactor=76`; parent `tenancy.events_created_idx` | - |


#### PostgreSQL

**Kind:** `partition`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Inherits:** `tenancy.events`

**Partition parent:** `tenancy.events`

**Partition bound:** `FOR VALUES FROM ('2025-01-01') TO ('2026-01-01')`



### `tenancy.special_events`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `bigint` | no | - | storage `plain`; inherited only |
| `payload` | `jsonb` | no | - | storage `extended`; inherited only |
| `category` | `text` | no | - | collation `pg_catalog."default"`; storage `extended` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `base_events_payload_not_null` | `not_null` | `payload` | `NOT NULL payload`; inherited |
| `base_events_tenant_id_not_null` | `not_null` | `tenant_id` | `NOT NULL tenant_id`; inherited |
| `special_events_category_not_null` | `not_null` | `category` | `NOT NULL category` |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`

**Inherits:** `tenancy.base_events`



### `type_system.measurements`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `token` | `type_system.scalar_token` | no | - | storage `plain` |
| `accepted` | `type_system.measurement_range` | no | - | storage `extended` |
| `historical` | `type_system.measurement_ranges` | no | - | storage `extended` |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| `measurements_accepted_not_null` | `not_null` | `accepted` | `NOT NULL accepted` |
| `measurements_historical_not_null` | `not_null` | `historical` | `NOT NULL historical` |
| `measurements_token_not_null` | `not_null` | `token` | `NOT NULL token` |


#### PostgreSQL

**Kind:** `table`

**Owner:** `dbmd`

**Persistence:** `permanent`

**Replica identity:** `default`

**Access method:** `heap`



## Views

### `audit.account_emails`

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

### `catalog.active_accounts`

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

### `secure.event_rollup`

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

### `secure.event_view`

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

## Triggers

### `audit.account_emails.account_emails_write`

**INSTEAD OF INSERT OR UPDATE OR DELETE** on `audit.account_emails`.

**Orientation:** `row`

**Function:** `audit.capture_row_change()`

**Enabled:** `origin`

**Arguments:** `view`

```sql
CREATE TRIGGER account_emails_write INSTEAD OF INSERT OR DELETE OR UPDATE ON audit.account_emails FOR EACH ROW EXECUTE FUNCTION audit.capture_row_change('view')
```

### `audit.accounts.accounts_balance_constraint`

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

### `audit.accounts.accounts_transition`

**AFTER UPDATE** on `audit.accounts`.

**Orientation:** `statement`

**Function:** `audit.capture_statement_change()`

**Enabled:** `disabled`

**Old transition table:** `previous_rows`

**New transition table:** `current_rows`

```sql
CREATE TRIGGER accounts_transition AFTER UPDATE ON audit.accounts REFERENCING OLD TABLE AS previous_rows NEW TABLE AS current_rows FOR EACH STATEMENT EXECUTE FUNCTION audit.capture_statement_change()
```

### `audit.accounts.accounts_truncate`

**AFTER TRUNCATE** on `audit.accounts`.

**Orientation:** `statement`

**Function:** `audit.capture_statement_change()`

**Enabled:** `replica`

```sql
CREATE TRIGGER accounts_truncate AFTER TRUNCATE ON audit.accounts FOR EACH STATEMENT EXECUTE FUNCTION audit.capture_statement_change()
```

### `audit.accounts.zz_accounts_change`

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

### `audit.partitioned_events.partitioned_events_change`

**BEFORE INSERT OR UPDATE** on `audit.partitioned_events`.

**Orientation:** `row`

**Function:** `audit.capture_row_change()`

**Enabled:** `origin`

**Arguments:** `partition`

```sql
CREATE TRIGGER partitioned_events_change BEFORE INSERT OR UPDATE ON audit.partitioned_events FOR EACH ROW EXECUTE FUNCTION audit.capture_row_change('partition')
```

### `audit.partitioned_events_2026.partitioned_events_change`

**BEFORE INSERT OR UPDATE** on `audit.partitioned_events_2026`.

**Orientation:** `row`

**Function:** `audit.capture_row_change()`

**Enabled:** `origin`

**Arguments:** `partition`

**Parent trigger:** `audit.partitioned_events.partitioned_events_change`

```sql
CREATE TRIGGER partitioned_events_change BEFORE INSERT OR UPDATE ON audit.partitioned_events_2026 FOR EACH ROW EXECUTE FUNCTION audit.capture_row_change('partition')
```

## Functions

### `advanced.capture_schema_change()`

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


### `aggregates.collect_integer(state integer[], value integer)`

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


### `aggregates.hypothetical_position(state integer[], hypothetical integer)`

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


### `aggregates.pick_integer(state integer[], fraction double precision)`

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


### `aggregates.total_combine(left_state bigint, right_state bigint)`

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


### `aggregates.total_final(state bigint)`

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


### `aggregates.total_inverse(state bigint, value integer)`

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


### `aggregates.total_step(state bigint, value integer)`

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


### `audit.capture_row_change()`

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


### `audit.capture_statement_change()`

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


### `infrastructure.fixture_btree_handler(internal)`

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


### `infrastructure.label_a_to_b(infrastructure.label_a)`

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


### `infrastructure.label_c_in(cstring)`

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


### `infrastructure.label_c_out(infrastructure.label_c)`

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


### `infrastructure.nonzero(integer)`

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


### `infrastructure.same_integer(integer, integer)`

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


### `routines.range_values(first_value integer, last_value integer)`

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


### `routines.row_number_clone()`

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


### `routines.starts_with(value text, prefix text)`

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


### `secure.event_count()`

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


### `type_system.scalar_token_in(cstring)`

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


### `type_system.scalar_token_out(type_system.scalar_token)`

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


## Procedures

### `infrastructure.accept_integer(IN value integer)`

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


### `secure.clear_events()`

**Arguments:** ``

**Owner:** `dbmd_acl_owner`

**Language:** `sql`

**Security:** `invoker`

```sql
CREATE OR REPLACE PROCEDURE secure.clear_events()
 LANGUAGE sql
AS $procedure$DELETE FROM secure.events$procedure$

```


## Aggregates

### `aggregates.hypothetical_position(integer ORDER BY integer)`

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



### `aggregates.integer_total(integer)`

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



### `aggregates.percentile_pick(double precision ORDER BY integer)`

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



### `secure.total_int(integer)`

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



## Casts

### `infrastructure.label_a AS infrastructure.label_b`

Fixture implicit cast

**Context:** `implicit`

**Method:** `function`

**Function:** `infrastructure.label_a_to_b(infrastructure.label_a)`



### `infrastructure.label_b AS infrastructure.label_c`

**Context:** `assignment`

**Method:** `input_output`



### `infrastructure.label_c AS integer`

**Context:** `explicit`

**Method:** `binary`



## Encoding Conversions

### `infrastructure.utf8_to_latin1`

Fixture default encoding conversion

**Owner:** `dbmd`

**Source encoding:** `UTF8`

**Target encoding:** `LATIN1`

**Function:** `pg_catalog.utf8_to_iso8859_1(integer,integer,pg_catalog.cstring,pg_catalog.internal,integer,boolean)`

**Default:** yes



## Operators

### `infrastructure.!!(NONE, integer)`

**Owner:** `dbmd`

**Kind:** `prefix`

**Result:** `boolean`

**Function:** `infrastructure.nonzero(integer)`

**Merge join:** no

**Hash join:** no



### `infrastructure.===(integer, integer)`

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



## Operator Families

### `infrastructure.integer_family`

Fixture integer operator family

**Owner:** `dbmd`

**Access method:** `btree`

**Operator:** strategy 1; `pg_catalog.<(integer,integer)` (`integer`, `integer`) via `btree`

**Operator:** strategy 2; `pg_catalog.<=(integer,integer)` (`integer`, `integer`) via `btree`

**Operator:** strategy 3; `pg_catalog.=(integer,integer)` (`integer`, `integer`) via `btree`

**Operator:** strategy 4; `pg_catalog.>=(integer,integer)` (`integer`, `integer`) via `btree`

**Operator:** strategy 5; `pg_catalog.>(integer,integer)` (`integer`, `integer`) via `btree`

**Support function:** number 1; `pg_catalog.btint4cmp(integer,integer)` (`integer`, `integer`)



## Operator Classes

### `infrastructure.integer_class`

Fixture integer operator class

**Owner:** `dbmd`

**Access method:** `btree`

**Family:** `infrastructure.integer_family`

**Input type:** `integer`

**Default:** no



## Access Methods

### `fixture_btree`

Fixture index access method

**Kind:** `index`

**Handler:** `infrastructure.fixture_btree_handler(pg_catalog.internal)`



## Procedural Languages

### `fixture_pl`

Fixture procedural language

**Owner:** `dbmd`

**Procedural:** yes

**Trusted:** yes

**Handler:** `pg_catalog.plpgsql_call_handler()`

**Inline handler:** `pg_catalog.plpgsql_inline_handler(pg_catalog.internal)`

**Validator:** `pg_catalog.plpgsql_validator(pg_catalog.oid)`



## Transforms

### `integer FOR fixture_pl`

Fixture integer transform

**Language:** `fixture_pl`

**From SQL:** `pg_catalog.textlike_support(pg_catalog.internal)`

**To SQL:** `pg_catalog.int4recv(pg_catalog.internal)`



## Rewrite Rules

### `advanced.orders.archive_order_delete`

Archives replicated deletes

**Event:** `delete`

**Instead:** no

**Enabled:** `replica`

```sql
CREATE RULE archive_order_delete AS
    ON DELETE TO advanced.orders DO  INSERT INTO advanced.deleted_orders (id)
  VALUES (old.id);
```


## Event Triggers

### `capture_schema_change`

Captures selected schema changes

**Owner:** `dbmd`

**Event:** `DDL command end`

**Function:** `advanced.capture_schema_change()`

**Enabled:** `always`

**Tags:** `CREATE TABLE, ALTER TABLE`

```sql
CREATE EVENT TRIGGER "capture_schema_change" ON ddl_command_end WHEN TAG IN ('CREATE TABLE', 'ALTER TABLE') EXECUTE FUNCTION advanced.capture_schema_change();
```


## Extended Statistics

### `advanced.orders_dependencies`

Cross-column order distribution

**Owner:** `dbmd`

**Kinds:** `ndistinct, dependencies, mcv`

**Statistics target:** 500

**Columns:** `customer_id, region`

```sql
CREATE STATISTICS advanced.orders_dependencies ON customer_id, region FROM advanced.orders
```


### `advanced.orders_expression`

**Owner:** `dbmd`

**Kinds:** `expressions`

**Statistics target:** -1

**Expression:** `lower(region)`

```sql
CREATE STATISTICS advanced.orders_expression ON lower(region) FROM advanced.orders
```


## Foreign-Data Wrappers

### `fixture_wrapper`

Fixture foreign-data wrapper

**Owner:** `dbmd`

**Option:** `api_token=<redacted>`

**Option:** `endpoint=catalog.example`



## Foreign Servers

### `fixture_server`

Fixture foreign server

**Owner:** `dbmd`

**Foreign-data wrapper:** `fixture_wrapper`

**Type:** `catalog`

**Version:** `1.0`

**Option:** `host=catalog.example`

**Option:** `password=<redacted>`



### `secure_server`

**Owner:** `dbmd_acl_owner`

**Foreign-data wrapper:** `postgres_fdw`

**Option:** `host=127.0.0.1`

**Option:** `dbname=postgres`



## User Mappings

### `PUBLIC ON fixture_server`

**Option:** `user=catalog_reader`

**Option:** `password=<redacted>`



## Text Search Parsers

### `advanced.default_parser`

Fixture parser backed by PostgreSQL defaults

**Start function:** `pg_catalog.prsd_start(pg_catalog.internal,integer)`

**Token function:** `pg_catalog.prsd_nexttoken(pg_catalog.internal,pg_catalog.internal,pg_catalog.internal)`

**End function:** `pg_catalog.prsd_end(pg_catalog.internal)`

**Headline function:** `pg_catalog.prsd_headline(pg_catalog.internal,pg_catalog.internal,pg_catalog.tsquery)`

**Token-types function:** `pg_catalog.prsd_lextype(pg_catalog.internal)`



## Text Search Templates

### `advanced.simple_template`

Fixture simple dictionary template

**Init function:** `pg_catalog.dsimple_init(pg_catalog.internal)`

**Lexize function:** `pg_catalog.dsimple_lexize(pg_catalog.internal,pg_catalog.internal,pg_catalog.internal,pg_catalog.internal)`



## Text Search Dictionaries

### `advanced.simple_dictionary`

Fixture stop-word dictionary

**Owner:** `dbmd`

**Template:** `advanced.simple_template`

**Options:** `stopwords = 'english'`



## Text Search Configurations

### `advanced.search_configuration`

Fixture search pipeline

**Owner:** `dbmd`

**Parser:** `advanced.default_parser`

**Mapping:** `asciiword`: `advanced.simple_dictionary, pg_catalog.english_stem`



## Publications

### `advanced_publication`

**Owner:** `dbmd`

**All tables:** no

**Actions:** `insert, update, delete, truncate`

**Generated columns:** `none`

**Publish via partition root:** no

**Table:** `advanced.orders`



### `all_tables`

**Owner:** `dbmd`

**All tables:** yes

**Actions:** `insert`

**Generated columns:** `none`

**Publish via partition root:** no



### `temporal_changes`

Stored generated values for analytics

**Owner:** `dbmd`

**All tables:** no

**Actions:** `insert`

**Generated columns:** `stored`

**Publish via partition root:** no

**Table:** `temporal.accounts`; columns `account_id, stored_amount`; where `base_amount >= 0`



### `temporal_schema`

**Owner:** `dbmd`

**All tables:** no

**Actions:** `insert, truncate`

**Generated columns:** `none`

**Publish via partition root:** no

**Schema:** `temporal`



## Subscriptions

### `advanced_subscription`

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



## Object Privileges

### `aggregate secure.total_int(integer) → PUBLIC EXECUTE`

**Object type:** `aggregate`

**Object:** `secure.total_int(integer)`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `PUBLIC`

**Privilege:** `EXECUTE`



### `aggregate secure.total_int(integer) → dbmd_acl_owner EXECUTE`

**Object type:** `aggregate`

**Object:** `secure.total_int(integer)`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `EXECUTE`



### `aggregate secure.total_int(integer) → dbmd_acl_reader EXECUTE`

**Object type:** `aggregate`

**Object:** `secure.total_int(integer)`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `EXECUTE`



### `database dbmd → PUBLIC CONNECT`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `PUBLIC`

**Privilege:** `CONNECT`



### `database dbmd → PUBLIC TEMPORARY`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `PUBLIC`

**Privilege:** `TEMPORARY`



### `database dbmd → dbmd CONNECT`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `CONNECT`



### `database dbmd → dbmd CREATE`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `CREATE`



### `database dbmd → dbmd TEMPORARY`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `TEMPORARY`



### `database dbmd → dbmd_acl_reader CONNECT`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `CONNECT`



### `database dbmd → dbmd_acl_reader CREATE`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `CREATE`



### `database dbmd → dbmd_acl_reader TEMPORARY`

**Object type:** `database`

**Object:** `dbmd`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `TEMPORARY`



### `foreign table secure.remote_events → dbmd_acl_owner DELETE`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `DELETE`



### `foreign table secure.remote_events → dbmd_acl_owner INSERT`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `INSERT`



### `foreign table secure.remote_events → dbmd_acl_owner MAINTAIN`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `MAINTAIN`



### `foreign table secure.remote_events → dbmd_acl_owner REFERENCES`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `REFERENCES`



### `foreign table secure.remote_events → dbmd_acl_owner SELECT`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



### `foreign table secure.remote_events → dbmd_acl_owner TRIGGER`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRIGGER`



### `foreign table secure.remote_events → dbmd_acl_owner TRUNCATE`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRUNCATE`



### `foreign table secure.remote_events → dbmd_acl_owner UPDATE`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



### `foreign table secure.remote_events → dbmd_acl_reader SELECT`

**Object type:** `foreign table`

**Object:** `secure.remote_events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



### `foreign-data wrapper postgres_fdw → dbmd USAGE`

**Object type:** `foreign-data wrapper`

**Object:** `postgres_fdw`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `USAGE`



### `foreign-data wrapper postgres_fdw → dbmd_acl_reader USAGE`

**Object type:** `foreign-data wrapper`

**Object:** `postgres_fdw`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



### `function secure.event_count() → PUBLIC EXECUTE`

**Object type:** `function`

**Object:** `secure.event_count()`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `PUBLIC`

**Privilege:** `EXECUTE`



### `function secure.event_count() → dbmd_acl_owner EXECUTE`

**Object type:** `function`

**Object:** `secure.event_count()`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `EXECUTE`



### `function secure.event_count() → dbmd_acl_reader EXECUTE`

**Object type:** `function`

**Object:** `secure.event_count()`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `EXECUTE`



### `language plpgsql → PUBLIC USAGE`

**Object type:** `language`

**Object:** `plpgsql`

**Grantor:** `dbmd`

**Grantee:** `PUBLIC`

**Privilege:** `USAGE`



### `language plpgsql → dbmd USAGE`

**Object type:** `language`

**Object:** `plpgsql`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `USAGE`



### `language plpgsql → dbmd_acl_reader USAGE`

**Object type:** `language`

**Object:** `plpgsql`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



### `large object 424242 → dbmd_acl_owner SELECT`

**Object type:** `large object`

**Object:** `424242`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



### `large object 424242 → dbmd_acl_owner UPDATE`

**Object type:** `large object`

**Object:** `424242`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



### `large object 424242 → dbmd_acl_reader SELECT`

**Object type:** `large object`

**Object:** `424242`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



### `large object 424242 → dbmd_acl_reader UPDATE`

**Object type:** `large object`

**Object:** `424242`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `UPDATE`



### `materialized view secure.event_rollup → dbmd_acl_owner DELETE`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `DELETE`



### `materialized view secure.event_rollup → dbmd_acl_owner INSERT`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `INSERT`



### `materialized view secure.event_rollup → dbmd_acl_owner MAINTAIN`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `MAINTAIN`



### `materialized view secure.event_rollup → dbmd_acl_owner REFERENCES`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `REFERENCES`



### `materialized view secure.event_rollup → dbmd_acl_owner SELECT`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



### `materialized view secure.event_rollup → dbmd_acl_owner TRIGGER`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRIGGER`



### `materialized view secure.event_rollup → dbmd_acl_owner TRUNCATE`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRUNCATE`



### `materialized view secure.event_rollup → dbmd_acl_owner UPDATE`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



### `materialized view secure.event_rollup → dbmd_acl_reader SELECT`

**Object type:** `materialized view`

**Object:** `secure.event_rollup`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



### `parameter statement_timeout → dbmd ALTER SYSTEM`

**Object type:** `parameter`

**Object:** `statement_timeout`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `ALTER SYSTEM`



### `parameter statement_timeout → dbmd SET`

**Object type:** `parameter`

**Object:** `statement_timeout`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `SET`



### `parameter statement_timeout → dbmd_acl_reader ALTER SYSTEM`

**Object type:** `parameter`

**Object:** `statement_timeout`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `ALTER SYSTEM`



### `parameter work_mem → dbmd ALTER SYSTEM`

**Object type:** `parameter`

**Object:** `work_mem`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `ALTER SYSTEM`



### `parameter work_mem → dbmd SET`

**Object type:** `parameter`

**Object:** `work_mem`

**Grantor:** `dbmd`

**Grantee:** `dbmd`

**Privilege:** `SET`



### `parameter work_mem → dbmd_acl_reader SET`

**Object type:** `parameter`

**Object:** `work_mem`

**Grantor:** `dbmd`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SET`



### `procedure secure.clear_events() → PUBLIC EXECUTE`

**Object type:** `procedure`

**Object:** `secure.clear_events()`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `PUBLIC`

**Privilege:** `EXECUTE`



### `procedure secure.clear_events() → dbmd_acl_owner EXECUTE`

**Object type:** `procedure`

**Object:** `secure.clear_events()`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `EXECUTE`



### `procedure secure.clear_events() → dbmd_acl_reader EXECUTE`

**Object type:** `procedure`

**Object:** `secure.clear_events()`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `EXECUTE`



### `schema public → PUBLIC USAGE`

**Object type:** `schema`

**Object:** `public`

**Grantor:** `pg_database_owner`

**Grantee:** `PUBLIC`

**Privilege:** `USAGE`



### `schema public → pg_database_owner CREATE`

**Object type:** `schema`

**Object:** `public`

**Grantor:** `pg_database_owner`

**Grantee:** `pg_database_owner`

**Privilege:** `CREATE`



### `schema public → pg_database_owner USAGE`

**Object type:** `schema`

**Object:** `public`

**Grantor:** `pg_database_owner`

**Grantee:** `pg_database_owner`

**Privilege:** `USAGE`



### `schema secure → dbmd_acl_owner CREATE`

**Object type:** `schema`

**Object:** `secure`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `CREATE`



### `schema secure → dbmd_acl_owner USAGE`

**Object type:** `schema`

**Object:** `secure`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `USAGE`



### `schema secure → dbmd_acl_reader USAGE`

**Object type:** `schema`

**Object:** `secure`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`

**Grant option:** yes



### `sequence secure.event_sequence → dbmd_acl_owner SELECT`

**Object type:** `sequence`

**Object:** `secure.event_sequence`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



### `sequence secure.event_sequence → dbmd_acl_owner UPDATE`

**Object type:** `sequence`

**Object:** `secure.event_sequence`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



### `sequence secure.event_sequence → dbmd_acl_owner USAGE`

**Object type:** `sequence`

**Object:** `secure.event_sequence`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `USAGE`



### `sequence secure.event_sequence → dbmd_acl_reader SELECT`

**Object type:** `sequence`

**Object:** `secure.event_sequence`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



### `sequence secure.event_sequence → dbmd_acl_reader UPDATE`

**Object type:** `sequence`

**Object:** `secure.event_sequence`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `UPDATE`



### `sequence secure.event_sequence → dbmd_acl_reader USAGE`

**Object type:** `sequence`

**Object:** `secure.event_sequence`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



### `foreign server secure_server → dbmd_acl_owner USAGE`

**Object type:** `foreign server`

**Object:** `secure_server`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `USAGE`



### `foreign server secure_server → dbmd_acl_reader USAGE`

**Object type:** `foreign server`

**Object:** `secure_server`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



### `table secure.events → dbmd_acl_owner DELETE`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `DELETE`



### `table secure.events → dbmd_acl_owner INSERT`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `INSERT`



### `table secure.events → dbmd_acl_owner MAINTAIN`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `MAINTAIN`



### `table secure.events → dbmd_acl_owner REFERENCES`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `REFERENCES`



### `table secure.events → dbmd_acl_owner SELECT`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



### `table secure.events → dbmd_acl_owner TRIGGER`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRIGGER`



### `table secure.events → dbmd_acl_owner TRUNCATE`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRUNCATE`



### `table secure.events → dbmd_acl_owner UPDATE`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



### `table secure.events → dbmd_acl_reader DELETE`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `DELETE`



### `table secure.events → dbmd_acl_reader INSERT`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `INSERT`



### `table secure.events → dbmd_acl_reader MAINTAIN`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `MAINTAIN`



### `table secure.events → dbmd_acl_reader REFERENCES`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `REFERENCES`



### `table secure.events → dbmd_acl_reader SELECT`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



### `table secure.events → dbmd_acl_reader TRIGGER`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `TRIGGER`



### `table secure.events → dbmd_acl_reader TRUNCATE`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `TRUNCATE`



### `table secure.events → dbmd_acl_reader UPDATE`

**Object type:** `table`

**Object:** `secure.events`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `UPDATE`



### `table column secure.events.payload → dbmd_acl_reader UPDATE`

**Object type:** `table column`

**Object:** `secure.events.payload`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `UPDATE`

**Grant option:** yes



### `type secure.event_code → PUBLIC USAGE`

**Object type:** `type`

**Object:** `secure.event_code`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `PUBLIC`

**Privilege:** `USAGE`



### `type secure.event_code → dbmd_acl_owner USAGE`

**Object type:** `type`

**Object:** `secure.event_code`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `USAGE`



### `type secure.event_code → dbmd_acl_reader USAGE`

**Object type:** `type`

**Object:** `secure.event_code`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



### `type secure.event_state → PUBLIC USAGE`

**Object type:** `type`

**Object:** `secure.event_state`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `PUBLIC`

**Privilege:** `USAGE`



### `type secure.event_state → dbmd_acl_owner USAGE`

**Object type:** `type`

**Object:** `secure.event_state`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `USAGE`



### `type secure.event_state → dbmd_acl_reader USAGE`

**Object type:** `type`

**Object:** `secure.event_state`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



### `view secure.event_view → dbmd_acl_owner DELETE`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `DELETE`



### `view secure.event_view → dbmd_acl_owner INSERT`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `INSERT`



### `view secure.event_view → dbmd_acl_owner MAINTAIN`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `MAINTAIN`



### `view secure.event_view → dbmd_acl_owner REFERENCES`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `REFERENCES`



### `view secure.event_view → dbmd_acl_owner SELECT`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



### `view secure.event_view → dbmd_acl_owner TRIGGER`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRIGGER`



### `view secure.event_view → dbmd_acl_owner TRUNCATE`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `TRUNCATE`



### `view secure.event_view → dbmd_acl_owner UPDATE`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



### `view secure.event_view → dbmd_acl_reader SELECT`

**Object type:** `view`

**Object:** `secure.event_view`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



## Default Privileges

### `dbmd_acl_owner / secure / sequences → dbmd_acl_reader USAGE`

**Owner:** `dbmd_acl_owner`

**Scope:** `secure`

**Object family:** `sequences`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



### `dbmd_acl_owner / secure / types → dbmd_acl_reader USAGE`

**Owner:** `dbmd_acl_owner`

**Scope:** `secure`

**Object family:** `types`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



### `dbmd_acl_owner / secure / routines → dbmd_acl_reader EXECUTE`

**Owner:** `dbmd_acl_owner`

**Scope:** `secure`

**Object family:** `routines`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `EXECUTE`



### `dbmd_acl_owner / secure / tables → dbmd_acl_reader SELECT`

**Owner:** `dbmd_acl_owner`

**Scope:** `secure`

**Object family:** `tables`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`

**Grant option:** yes



### `dbmd_acl_owner / database-wide / large objects → dbmd_acl_owner SELECT`

**Owner:** `dbmd_acl_owner`

**Scope:** `database-wide`

**Object family:** `large objects`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `SELECT`



### `dbmd_acl_owner / database-wide / large objects → dbmd_acl_owner UPDATE`

**Owner:** `dbmd_acl_owner`

**Scope:** `database-wide`

**Object family:** `large objects`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `UPDATE`



### `dbmd_acl_owner / database-wide / large objects → dbmd_acl_reader SELECT`

**Owner:** `dbmd_acl_owner`

**Scope:** `database-wide`

**Object family:** `large objects`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `SELECT`



### `dbmd_acl_owner / database-wide / schemas → dbmd_acl_owner CREATE`

**Owner:** `dbmd_acl_owner`

**Scope:** `database-wide`

**Object family:** `schemas`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `CREATE`



### `dbmd_acl_owner / database-wide / schemas → dbmd_acl_owner USAGE`

**Owner:** `dbmd_acl_owner`

**Scope:** `database-wide`

**Object family:** `schemas`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_owner`

**Privilege:** `USAGE`



### `dbmd_acl_owner / database-wide / schemas → dbmd_acl_reader USAGE`

**Owner:** `dbmd_acl_owner`

**Scope:** `database-wide`

**Object family:** `schemas`

**Grantor:** `dbmd_acl_owner`

**Grantee:** `dbmd_acl_reader`

**Privilege:** `USAGE`



## Large Objects

### `424242`

Fixture document payload

**Owner:** `dbmd_acl_owner`

**Contents:** `omitted`



## Collations

### `temporal.unicode_fast`

Fast Unicode semantics

**Owner:** `dbmd`

**Provider:** `builtin`

**Deterministic:** yes

**Encoding:** `UTF8`

**Locale:** `PG_UNICODE_FAST`

**Version:** `1`



## Extensions

### `btree_gist`

Temporal exclusion operator support

**Owner:** `dbmd`

**Schema:** `public`

**Version:** `1.8`

**Relocatable:** yes

**Owned objects:** 288 (`function`: 212, `operator`: 12, `operator class`: 26, `operator family`: 26, `type`: 12)



### `plpgsql`

PL/pgSQL procedural language

**Owner:** `dbmd`

**Schema:** `pg_catalog`

**Version:** `1.0`

**Relocatable:** no

**Owned objects:** 4 (`function`: 3, `language`: 1)



### `postgres_fdw`

foreign-data wrapper for remote PostgreSQL servers

**Owner:** `dbmd`

**Schema:** `public`

**Version:** `1.2`

**Relocatable:** yes

**Owned objects:** 6 (`foreign-data wrapper`: 1, `function`: 5)



