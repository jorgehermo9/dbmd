# `secure.event_sequence`

**Owner:** `dbmd_acl_owner`

**Type:** `bigint`

**Start:** 1

**Minimum:** 1

**Maximum:** 9223372036854775807

**Increment:** 1

**Cache:** 1

**Cycle:** no

**Persistence:** `permanent`

```sql
CREATE SEQUENCE "secure"."event_sequence" AS bigint INCREMENT BY 1 MINVALUE 1 MAXVALUE 9223372036854775807 START WITH 1 CACHE 1 NO CYCLE OWNED BY NONE;
```
