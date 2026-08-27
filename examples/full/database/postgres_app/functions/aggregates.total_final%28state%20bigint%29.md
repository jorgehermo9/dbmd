# `aggregates.total_final(state bigint)`

**Kind:** `ordinary`

**Arguments:** `state bigint`

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
CREATE OR REPLACE FUNCTION aggregates.total_final(state bigint)
 RETURNS bigint
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE
RETURN state

```
