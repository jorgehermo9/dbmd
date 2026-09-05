# `advanced.capture_schema_change()`

**Kind:** `ordinary`

**Arguments:** ``

**Returns:** `event_trigger`

**Owner:** `dbmd`

**Language:** `plpgsql`

**Volatility:** `volatile`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** called

**Leakproof:** no

**Returns set:** no

**Cost:** 100

```sql
CREATE OR REPLACE FUNCTION advanced.capture_schema_change()
 RETURNS event_trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    NULL;
END;
$function$

```
