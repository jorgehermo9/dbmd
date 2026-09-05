# `aggregates.total_combine(left_state bigint, right_state bigint)`

**Kind:** `ordinary`

**Arguments:** `left_state bigint, right_state bigint`

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
CREATE OR REPLACE FUNCTION aggregates.total_combine(left_state bigint, right_state bigint)
 RETURNS bigint
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE
RETURN (COALESCE(left_state, (0)::bigint) + COALESCE(right_state, (0)::bigint))

```
