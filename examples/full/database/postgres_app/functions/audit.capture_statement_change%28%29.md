# `audit.capture_statement_change()`

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
CREATE OR REPLACE FUNCTION audit.capture_statement_change()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    RETURN NULL;
END;
$function$

```
