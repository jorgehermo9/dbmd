# `audit.account_emails.account_emails_write`

**INSTEAD OF INSERT OR UPDATE OR DELETE** on `audit.account_emails`.

**Orientation:** `row`

**Function:** `audit.capture_row_change()`

**Enabled:** `origin`

**Arguments:** `view`

```sql
CREATE TRIGGER account_emails_write INSTEAD OF INSERT OR DELETE OR UPDATE ON audit.account_emails FOR EACH ROW EXECUTE FUNCTION audit.capture_row_change('view')
```