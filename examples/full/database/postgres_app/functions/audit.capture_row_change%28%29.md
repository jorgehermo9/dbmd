# `audit.capture_row_change()`

**Kind:** `ordinary`

**Arguments:** ``

**Returns:** `trigger`

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
CREATE OR REPLACE FUNCTION audit.capture_row_change()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    RETURN COALESCE(NEW, OLD);
END;
$function$

```
