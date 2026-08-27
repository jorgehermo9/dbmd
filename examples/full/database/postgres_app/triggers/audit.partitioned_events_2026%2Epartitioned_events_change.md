# `audit.partitioned_events_2026.partitioned_events_change`

**BEFORE INSERT OR UPDATE** on `audit.partitioned_events_2026`.

**Orientation:** `row`

**Function:** `audit.capture_row_change()`

**Enabled:** `origin`

**Arguments:** `partition`

**Parent trigger:** `audit.partitioned_events.partitioned_events_change`

```sql
CREATE TRIGGER partitioned_events_change BEFORE INSERT OR UPDATE ON audit.partitioned_events_2026 FOR EACH ROW EXECUTE FUNCTION audit.capture_row_change('partition')
```