# `main.account_balances`

| Column | Type | Nullable |
|---|---|---|
| `id` | `INTEGER` | unknown |
| `balance_cents` | `INTEGER` | unknown |


```sql
CREATE VIEW account_balances AS
SELECT id, balance_cents FROM accounts
```