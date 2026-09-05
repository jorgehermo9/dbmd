# `infrastructure.accept_integer(IN value integer)`

**Arguments:** `IN value integer`

**Owner:** `dbmd`

**Language:** `fixture_pl`

**Security:** `invoker`

**Transform:** `integer`

```sql
CREATE OR REPLACE PROCEDURE infrastructure.accept_integer(IN value integer)
 TRANSFORM FOR TYPE integer
 LANGUAGE fixture_pl
AS $procedure$
BEGIN
    NULL;
END;
$procedure$

```
