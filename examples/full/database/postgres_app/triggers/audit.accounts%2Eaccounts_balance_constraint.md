# `audit.accounts.accounts_balance_constraint`

**AFTER UPDATE OF balance** on `audit.accounts`.

**Orientation:** `row`

**Function:** `audit.capture_row_change()`

**Enabled:** `origin`

**Arguments:** `balance`

**Constraint trigger:** deferrable initially deferred; from `audit.account_limits`

When: `new.balance < 0`

```sql
CREATE CONSTRAINT TRIGGER accounts_balance_constraint AFTER UPDATE OF balance ON audit.accounts FROM audit.account_limits DEFERRABLE INITIALLY DEFERRED FOR EACH ROW WHEN (new.balance < 0) EXECUTE FUNCTION audit.capture_row_change('balance')
```