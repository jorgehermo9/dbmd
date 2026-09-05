# `advanced.orders.archive_order_delete`

Archives replicated deletes

**Event:** `delete`

**Instead:** no

**Enabled:** `replica`

```sql
CREATE RULE archive_order_delete AS
    ON DELETE TO advanced.orders DO  INSERT INTO advanced.deleted_orders (id)
  VALUES (old.id);
```
