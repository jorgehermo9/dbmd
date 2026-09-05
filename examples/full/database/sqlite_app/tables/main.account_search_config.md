# `main.account_search_config`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `k` | `` | no | - |  |
| `v` | `` | yes | - |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `k` | - |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `sqlite_autoindex_account_search_config_1` | `k` ascending collate `BINARY` | yes | `primary_key` | - |


## SQLite

**Kind:** `shadow` owned by `account_search`

Without rowid.

```sql
CREATE TABLE 'account_search_config'(k PRIMARY KEY, v) WITHOUT ROWID
```
