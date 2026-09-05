# `routines.starts_with(value text, prefix text)`

Planner-supported strict and leakproof function

**Kind:** `ordinary`

**Arguments:** `value text, prefix text DEFAULT ''::text`

**Returns:** `boolean`

**Owner:** `dbmd`

**Language:** `internal`

**Volatility:** `immutable`

**Parallel:** `safe`

**Security:** `invoker`

**Null input:** strict

**Leakproof:** yes

**Returns set:** no

**Cost:** 3

**Support function:** `pg_catalog.text_starts_with_support(pg_catalog.internal)`

```sql
CREATE OR REPLACE FUNCTION routines.starts_with(value text, prefix text DEFAULT ''::text)
 RETURNS boolean
 LANGUAGE internal
 IMMUTABLE PARALLEL SAFE STRICT LEAKPROOF COST 3 SUPPORT text_starts_with_support
AS $function$text_starts_with$function$

```
