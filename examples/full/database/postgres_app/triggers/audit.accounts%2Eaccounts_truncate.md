# `audit.accounts.accounts_truncate`

**AFTER TRUNCATE** on `audit.accounts`.

**Orientation:** `statement`

**Function:** `audit.capture_statement_change()`

**Enabled:** `replica`

```sql
CREATE TRIGGER accounts_truncate AFTER TRUNCATE ON audit.accounts FOR EACH STATEMENT EXECUTE FUNCTION audit.capture_statement_change()
```