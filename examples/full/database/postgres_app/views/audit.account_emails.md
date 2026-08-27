# `audit.account_emails`

Writable account email projection

**Kind:** `view`

**Owner:** `dbmd`

**Persistence:** `permanent`

| Column | Type | Nullable |
|---|---|---|
| `id` | `bigint` | yes |
| `email` | `text` | yes |


```sql
 SELECT id,
    email
   FROM audit.accounts;
```