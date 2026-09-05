# `main.account_search`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `email` | `` | yes | - |  |
| `account_search` | `` | yes | - | virtual_table_hidden |
| `rank` | `` | yes | - | virtual_table_hidden |


## SQLite

**Kind:** `virtual` using `fts5` with arguments `email, content='accounts', content_rowid='id'`

```sql
CREATE VIRTUAL TABLE account_search USING fts5(email, content='accounts', content_rowid='id')
```
