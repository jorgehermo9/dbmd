# `routines.range_values(first_value integer, last_value integer)`

**Kind:** `ordinary`

**Arguments:** `first_value integer, last_value integer`

**Returns:** `SETOF integer`

**Owner:** `dbmd`

**Language:** `sql`

**Volatility:** `stable`

**Parallel:** `restricted`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** yes

**Cost:** 7

**Rows:** 25

**Setting:** `search_path=pg_catalog`

```sql
CREATE OR REPLACE FUNCTION routines.range_values(first_value integer, last_value integer)
 RETURNS SETOF integer
 LANGUAGE sql
 STABLE PARALLEL RESTRICTED COST 7 ROWS 25
 SET search_path TO 'pg_catalog'
AS $function$ SELECT generate_series(first_value, last_value) $function$

```
