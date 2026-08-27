# `analytics.remote_accounts`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `account_id` | `UInt64` | no | - | precision 64 base 2; scale 0 |
| `email` | `String` | no | - |  |


## ClickHouse

**Kind:** `table`

**UUID:** `20000000-0000-0000-0000-000000000007`

**Engine:** `MySQL('127.0.0.1:3306', 'remote_app', 'accounts', 'remote_reader', '[HIDDEN]')`

**Engine argument:** 1: `'127.0.0.1:3306'`

**Engine argument:** 2: `'remote_app'`

**Engine argument:** 3: `'accounts'`

**Engine argument:** 4: `'remote_reader'`

**Engine argument:** 5: `'[HIDDEN]'`

```sql
CREATE TABLE analytics.remote_accounts (`account_id` UInt64, `email` String) ENGINE = MySQL('127.0.0.1:3306', 'remote_app', 'accounts', 'remote_reader', '[HIDDEN]')
```
