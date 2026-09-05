# `main.accounts.accounts_normalize_email`

**AFTER UPDATE OF email** on `main.accounts`.

When: `NEW.email <> lower (NEW.email)`

```sql
CREATE TRIGGER accounts_normalize_email
AFTER UPDATE OF email ON accounts
WHEN NEW.email != lower(NEW.email)
BEGIN
    UPDATE accounts SET email = lower(NEW.email) WHERE id = NEW.id;
END
```