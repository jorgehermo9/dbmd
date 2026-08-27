# `warehouse.analytics.accounts_for_tenant`

**Kind:** `table macro`

**Return type:** `-`

**Parameters:** `owner_id`

**Side effects:** unknown

```sql
SELECT * FROM analytics.accounts WHERE (tenant_id = owner_id)
```
