# `type_system.scalar_token_out(type_system.scalar_token)`

**Kind:** `ordinary`

**Arguments:** `type_system.scalar_token`

**Returns:** `cstring`

**Owner:** `dbmd`

**Language:** `internal`

**Volatility:** `immutable`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** strict

**Leakproof:** no

**Returns set:** no

**Cost:** 1

```sql
CREATE OR REPLACE FUNCTION type_system.scalar_token_out(type_system.scalar_token)
 RETURNS cstring
 LANGUAGE internal
 IMMUTABLE STRICT
AS $function$int4out$function$

```
