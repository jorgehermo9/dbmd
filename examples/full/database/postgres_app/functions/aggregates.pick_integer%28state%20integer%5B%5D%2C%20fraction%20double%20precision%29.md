# `aggregates.pick_integer(state integer[], fraction double precision)`

**Kind:** `ordinary`

**Arguments:** `state integer[], fraction double precision`

**Returns:** `integer`

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
CREATE OR REPLACE FUNCTION aggregates.pick_integer(state integer[], fraction double precision)
 RETURNS integer
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE
RETURN (SELECT value.value FROM unnest(pick_integer.state) value(value) ORDER BY value.value OFFSET LEAST((cardinality(pick_integer.state) - 1), GREATEST(0, (floor((pick_integer.fraction * ((cardinality(pick_integer.state) - 1))::double precision)))::integer)) LIMIT 1)

```
