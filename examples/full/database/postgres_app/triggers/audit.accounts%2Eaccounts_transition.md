# `audit.accounts.accounts_transition`

**AFTER UPDATE** on `audit.accounts`.

**Orientation:** `statement`

**Function:** `audit.capture_statement_change()`

**Enabled:** `disabled`

**Old transition table:** `previous_rows`

**New transition table:** `current_rows`

```sql
CREATE TRIGGER accounts_transition AFTER UPDATE ON audit.accounts REFERENCING OLD TABLE AS previous_rows NEW TABLE AS current_rows FOR EACH STATEMENT EXECUTE FUNCTION audit.capture_statement_change()
```