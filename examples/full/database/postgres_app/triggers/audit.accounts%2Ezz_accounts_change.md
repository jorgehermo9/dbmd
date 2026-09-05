# `audit.accounts.zz_accounts_change`

Captures relevant account row changes

**BEFORE INSERT OR UPDATE OF email, balance OR DELETE** on `audit.accounts`.

**Orientation:** `row`

**Function:** `audit.capture_row_change()`

**Enabled:** `always`

**Arguments:** `history, full`

When: `pg_trigger_depth() = 0`

```sql
CREATE TRIGGER zz_accounts_change BEFORE INSERT OR DELETE OR UPDATE OF email, balance ON audit.accounts FOR EACH ROW WHEN (pg_trigger_depth() = 0) EXECUTE FUNCTION audit.capture_row_change('history', 'full')
```