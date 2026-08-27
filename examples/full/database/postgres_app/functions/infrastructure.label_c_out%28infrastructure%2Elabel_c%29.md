# `infrastructure.label_c_out(infrastructure.label_c)`

**Kind:** `ordinary`

**Arguments:** `infrastructure.label_c`

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
CREATE OR REPLACE FUNCTION infrastructure.label_c_out(infrastructure.label_c)
 RETURNS cstring
 LANGUAGE internal
 IMMUTABLE STRICT
AS $function$int4out$function$

```
