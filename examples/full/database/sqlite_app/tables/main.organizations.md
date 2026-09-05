# `main.organizations`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `tenant_id` | `INTEGER` | no | - |  |
| `slug` | `TEXT` | no | - | collate `NOCASE` |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `not_null` | `tenant_id` | - |
| - | `not_null` | `slug` | - |
| `organizations_pk` | `primary_key` | `tenant_id, slug` | - |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `sqlite_autoindex_organizations_1` | `tenant_id` ascending collate `BINARY`, `slug` ascending collate `NOCASE` | yes | `primary_key` | - |


## SQLite

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
