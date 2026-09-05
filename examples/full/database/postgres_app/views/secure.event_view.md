# `secure.event_view`

**Kind:** `view`

**Owner:** `dbmd_acl_owner`

**Persistence:** `permanent`

| Column | Type | Nullable |
|---|---|---|
| `id` | `bigint` | yes |
| `payload` | `text` | yes |


```sql
 SELECT id,
    payload
   FROM secure.events;
```