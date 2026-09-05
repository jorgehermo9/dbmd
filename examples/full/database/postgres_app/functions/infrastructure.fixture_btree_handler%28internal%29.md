# `infrastructure.fixture_btree_handler(internal)`

**Kind:** `ordinary`

**Arguments:** `internal`

**Returns:** `index_am_handler`

**Owner:** `dbmd`

**Language:** `internal`

**Volatility:** `volatile`

**Parallel:** `unsafe`

**Security:** `invoker`

**Null input:** strict

**Leakproof:** no

**Returns set:** no

**Cost:** 1

```sql
CREATE OR REPLACE FUNCTION infrastructure.fixture_btree_handler(internal)
 RETURNS index_am_handler
 LANGUAGE internal
 STRICT
AS $function$bthandler$function$

```
