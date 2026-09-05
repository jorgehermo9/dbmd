# `infrastructure.same_integer(integer, integer)`

**Kind:** `ordinary`

**Arguments:** `integer, integer`

**Returns:** `boolean`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `immutable`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** strict

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION infrastructure.same_integer(integer, integer)
 RETURNS boolean
 LANGUAGE sql
 IMMUTABLE STRICT
RETURN ($1 = $2)

```
