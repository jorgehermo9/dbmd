# `secure.clear_events()`

**Arguments:** ``

**Owner:** `dbmd_acl_owner`

**Language:** `sql`

**Security:** `invoker`

```sql
CREATE OR REPLACE PROCEDURE secure.clear_events()
 LANGUAGE sql
AS $procedure$DELETE FROM secure.events$procedure$

```
