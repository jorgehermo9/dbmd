# `main.accounts`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `account_id` | `INTEGER` | no | - |  |
| `organization_id` | `INTEGER` | no | - |  |
| `email` | `TEXT` | no | - | collate `NOCASE` |
| `normalized_email` | `TEXT` | yes | - | stored_generated; as `lower (email)` |
| `status` | `TEXT` | no | `'active'` |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `account_id` | - |
| - | `not_null` | `organization_id` | - |
| - | `not_null` | `email` | - |
| - | `not_null` | `status` | - |
| - | `check` | `status` | `status IN ('active', 'disabled')` |
| - | `foreign_key` | `organization_id` | references `main.organizations(organization_id)`; update `cascade`, delete `restrict`; not deferrable |
| - | `unique` | `organization_id, email` | - |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `accounts_active_email_idx` | `normalized_email` ascending collate `BINARY` | no | `create_index` | `status = 'active'` |
| `sqlite_autoindex_accounts_1` | `organization_id` ascending collate `BINARY`, `email` ascending collate `NOCASE` | yes | `unique_constraint` | - |


## SQLite

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
