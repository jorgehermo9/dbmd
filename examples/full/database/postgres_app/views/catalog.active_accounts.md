# `catalog.active_accounts`

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
   FROM catalog.accounts
  WHERE state = 'active'::catalog.account_state;
```