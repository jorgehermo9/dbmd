# `main.account_search_docsize`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `INTEGER` | no | - |  |
| `sz` | `BLOB` | yes | - |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `id` | - |


## SQLite

**Kind:** `shadow` owned by `account_search`

```sql
CREATE TABLE 'account_search_docsize'(id INTEGER PRIMARY KEY, sz BLOB)
```
