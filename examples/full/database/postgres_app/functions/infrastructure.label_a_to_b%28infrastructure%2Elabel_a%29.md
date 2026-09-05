# `infrastructure.label_a_to_b(infrastructure.label_a)`

**Kind:** `ordinary`

**Arguments:** `infrastructure.label_a`

**Returns:** `infrastructure.label_b`

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
CREATE OR REPLACE FUNCTION infrastructure.label_a_to_b(infrastructure.label_a)
 RETURNS infrastructure.label_b
 LANGUAGE sql
 IMMUTABLE STRICT
RETURN (($1)::text)::infrastructure.label_b

```
