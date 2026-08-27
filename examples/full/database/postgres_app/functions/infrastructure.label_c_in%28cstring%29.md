# `infrastructure.label_c_in(cstring)`

**Kind:** `ordinary`

**Arguments:** `cstring`

**Returns:** `infrastructure.label_c`

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
CREATE OR REPLACE FUNCTION infrastructure.label_c_in(cstring)
 RETURNS infrastructure.label_c
 LANGUAGE internal
 IMMUTABLE STRICT
AS $function$int4in$function$

```
