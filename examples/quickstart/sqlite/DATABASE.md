# Database: `Application database`

## Namespaces

| Name | Comment |
|---|---|
| `main` | - |


## Tables

### `main.accounts`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `account_id` | `INTEGER` | no | - |  |
| `organization_id` | `INTEGER` | no | - |  |
| `email` | `TEXT` | no | - | collate `NOCASE` |
| `normalized_email` | `TEXT` | yes | - | stored_generated; as `lower (email)` |
| `status` | `TEXT` | no | `'active'` |  |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `account_id` | - |
| - | `not_null` | `organization_id` | - |
| - | `not_null` | `email` | - |
| - | `not_null` | `status` | - |
| - | `check` | `status` | `status IN ('active', 'disabled')` |
| - | `foreign_key` | `organization_id` | references `main.organizations(organization_id)`; update `cascade`, delete `restrict`; not deferrable |
| - | `unique` | `organization_id, email` | - |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_active_email_idx` | `normalized_email` ascending collate `BINARY` | no | `create_index` | `status = 'active'` |
| `sqlite_autoindex_accounts_1` | `organization_id` ascending collate `BINARY`, `email` ascending collate `NOCASE` | yes | `unique_constraint` | - |


#### SQLite

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


### `main.organizations`

#### Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `organization_id` | `INTEGER` | no | - |  |
| `slug` | `TEXT` | no | - |  |
| `name` | `TEXT` | no | - |  |


#### Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `organization_id` | - |
| - | `not_null` | `slug` | - |
| - | `unique` | `slug` | - |
| - | `not_null` | `name` | - |


#### Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `sqlite_autoindex_organizations_1` | `slug` ascending collate `BINARY` | yes | `unique_constraint` | - |


#### SQLite

**Kind:** `ordinary`

Strict table.

```sql
CREATE TABLE organizations (
    organization_id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
) STRICT
```


## Views

### `main.active_accounts`

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

