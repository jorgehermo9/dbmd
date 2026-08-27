# `main.account_search_idx`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `segid` | `` | no | - |  |
| `term` | `` | no | - |  |
| `pgno` | `` | yes | - |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `segid, term` | - |


## Indexes

| Name | Terms | Unique | Origin / details | Predicate |
|---|---|---|---|---|
| `sqlite_autoindex_account_search_idx_1` | `segid` ascending collate `BINARY`, `term` ascending collate `BINARY` | yes | `primary_key` | - |


## SQLite

**Kind:** `shadow` owned by `account_search`

Without rowid.

```sql
CREATE TABLE 'account_search_idx'(segid, term, pgno, PRIMARY KEY(segid, term)) WITHOUT ROWID
```
