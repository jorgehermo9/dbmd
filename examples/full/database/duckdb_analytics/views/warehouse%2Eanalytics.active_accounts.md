# `warehouse.analytics.active_accounts`

Active accounts only

**Temporary:** no

| Column | Type | Nullable |
|---|---|---|


```sql
CREATE VIEW analytics.active_accounts AS SELECT tenant_id, account_id, email FROM analytics.accounts WHERE (status = 'active');
```