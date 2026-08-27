# `main.organizations`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `organization_id` | `INTEGER` | no | - |  |
| `slug` | `TEXT` | no | - |  |
| `name` | `TEXT` | no | - |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `organization_id` | - |
| - | `not_null` | `slug` | - |
| - | `unique` | `slug` | - |
| - | `not_null` | `name` | - |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `sqlite_autoindex_organizations_1` | `slug` ascending collate `BINARY` | yes | `unique_constraint` | - |


## SQLite

**Kind:** `ordinary`

Strict table.

```sql
CREATE TABLE organizations (
    organization_id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
) STRICT
```
