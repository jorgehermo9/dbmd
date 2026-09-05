# `main.account_search_data`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `INTEGER` | no | - |  |
| `block` | `BLOB` | yes | - |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `id` | - |


## SQLite

**Kind:** `shadow` owned by `account_search`

```sql
CREATE TABLE 'account_search_data'(id INTEGER PRIMARY KEY, block BLOB)
```
