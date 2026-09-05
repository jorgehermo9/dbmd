# `secure.event_rollup`

**Kind:** `materialized_view`

**Owner:** `dbmd_acl_owner`

**Persistence:** `permanent`

**Populated:** no

**Access method:** `heap`

| Column | Type | Nullable |
|---|---|---|
| `event_count` | `bigint` | yes |


```sql
 SELECT count(*) AS event_count
   FROM secure.events;
```