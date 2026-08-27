# `main.account_directory`

| Column | Type | Nullable |
|---|---|---|
| `account_id` | `INTEGER` | unknown |
| `email` | `TEXT` | unknown |


```sql
CREATE VIEW account_directory (account_id, email) AS
SELECT id, email FROM accounts
```