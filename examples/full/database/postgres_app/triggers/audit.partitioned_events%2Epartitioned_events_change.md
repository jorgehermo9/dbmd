# `audit.partitioned_events.partitioned_events_change`

**BEFORE INSERT OR UPDATE** on `audit.partitioned_events`.

**Orientation:** `row`

**Function:** `audit.capture_row_change()`

**Enabled:** `origin`

**Arguments:** `partition`

```sql
CREATE TRIGGER partitioned_events_change BEFORE INSERT OR UPDATE ON audit.partitioned_events FOR EACH ROW EXECUTE FUNCTION audit.capture_row_change('partition')
```