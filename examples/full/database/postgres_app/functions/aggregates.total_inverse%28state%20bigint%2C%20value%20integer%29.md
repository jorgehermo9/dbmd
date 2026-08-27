# `aggregates.total_inverse(state bigint, value integer)`

**Kind:** `ordinary`

**Arguments:** `state bigint, value integer`

**Returns:** `bigint`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `immutable`

**Parallel:** `safe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION aggregates.total_inverse(state bigint, value integer)
 RETURNS bigint
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE
RETURN (COALESCE(state, (0)::bigint) - COALESCE(value, 0))

```
