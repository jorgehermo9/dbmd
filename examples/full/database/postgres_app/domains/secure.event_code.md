# `secure.event_code`

**Base type:** `text`

**Nullable:** yes

**Owner:** `dbmd_acl_owner`

**Collation:** `pg_catalog."default"`

**Constraint:** `event_code_check`: `CHECK (VALUE <> ''::text)`

```sql
CREATE DOMAIN "secure"."event_code"
    AS text
    COLLATE pg_catalog."default"
    CONSTRAINT "event_code_check" CHECK (VALUE <> ''::text);
```
