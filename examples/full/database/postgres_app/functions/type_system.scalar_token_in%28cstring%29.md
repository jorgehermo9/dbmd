# `type_system.scalar_token_in(cstring)`

**Kind:** `ordinary`

**Arguments:** `cstring`

**Returns:** `type_system.scalar_token`

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
CREATE OR REPLACE FUNCTION type_system.scalar_token_in(cstring)
 RETURNS type_system.scalar_token
 LANGUAGE internal
 IMMUTABLE STRICT
AS $function$int4in$function$

```
