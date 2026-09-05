# `aggregates.collect_integer(state integer[], value integer)`

**Kind:** `ordinary`

**Arguments:** `state integer[], value integer`

**Returns:** `integer[]`

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
CREATE OR REPLACE FUNCTION aggregates.collect_integer(state integer[], value integer)
 RETURNS integer[]
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE
RETURN array_append(state, value)

```
