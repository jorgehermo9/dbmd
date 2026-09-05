# `main.active_accounts`

| Column | Type | Nullable |
|---|---|---|
| `account_id` | `INTEGER` | unknown |
| `organization_id` | `INTEGER` | unknown |
| `email` | `TEXT` | unknown |


```sql
CREATE VIEW active_accounts AS
SELECT account_id, organization_id, email
FROM accounts
WHERE status = 'active'
```