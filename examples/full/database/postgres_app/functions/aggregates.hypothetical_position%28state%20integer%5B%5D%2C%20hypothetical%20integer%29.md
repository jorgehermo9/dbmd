# `aggregates.hypothetical_position(state integer[], hypothetical integer)`

**Kind:** `ordinary`

**Arguments:** `state integer[], hypothetical integer`

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
CREATE OR REPLACE FUNCTION aggregates.hypothetical_position(state integer[], hypothetical integer)
 RETURNS bigint
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE
RETURN (SELECT (1 + count(*)) FROM unnest(hypothetical_position.state) value(value) WHERE (value.value < hypothetical_position.hypothetical))

```
