# `storage.device_row`

**Owner:** `dbmd`

**Attribute:** `device_id` `bigint`

**Attribute:** `payload` `text`; collation `pg_catalog."default"`

```sql
CREATE TYPE "storage"."device_row" AS (
    "device_id" bigint,
    "payload" text COLLATE pg_catalog."default"
);
```
