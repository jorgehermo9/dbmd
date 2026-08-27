# `secure.event_count()`

**Kind:** `ordinary`

**Arguments:** ``

**Returns:** `bigint`

**Owner:** `dbmd_acl_owner`

**Language:** `sql`

**Volatility:** `volatile`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION secure.event_count()
 RETURNS bigint
 LANGUAGE sql
RETURN (SELECT count(*) AS count FROM secure.events)

```
