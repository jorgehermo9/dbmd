# `main.migration_target`

## Columns

| Column | Type | Nullable | Default | Notes |
|---|---|---|---|---|
| `id` | `INTEGER` | no | - |  |
| `name` | `TEXT` | no | - |  |
| `generated_name` | `TEXT` | yes | - | virtual_generated; as `upper (name)` |
| `optional_note` | `TEXT` | yes | - |  |


## Constraints

| Name | Kind | Columns | Details |
|---|---|---|---|
| - | `primary_key` | `id` | - |
| - | `not_null` | `name` | - |


## SQLite

**Kind:** `ordinary`

```sql
CREATE TABLE "migration_target" (id INTEGER PRIMARY KEY, name TEXT NOT NULL, generated_name TEXT AS (upper(name)) VIRTUAL, optional_note TEXT)
```
