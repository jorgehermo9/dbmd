# `capture_schema_change`

Captures selected schema changes

**Owner:** `dbmd`

**Event:** `DDL command end`

**Function:** `advanced.capture_schema_change()`

**Enabled:** `always`

**Tags:** `CREATE TABLE, ALTER TABLE`

```sql
CREATE EVENT TRIGGER "capture_schema_change" ON ddl_command_end WHEN TAG IN ('CREATE TABLE', 'ALTER TABLE') EXECUTE FUNCTION advanced.capture_schema_change();
```
