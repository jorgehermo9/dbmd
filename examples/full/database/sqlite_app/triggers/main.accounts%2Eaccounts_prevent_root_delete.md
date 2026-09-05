# `main.accounts.accounts_prevent_root_delete`

**BEFORE DELETE** on `main.accounts`.

When: `OLD.id = 0`

```sql
CREATE TRIGGER accounts_prevent_root_delete
BEFORE DELETE ON accounts
WHEN OLD.id = 0
BEGIN
    SELECT RAISE(IGNORE);
END
```